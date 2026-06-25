pub mod camera;
pub mod debug_shape;
pub mod selection;
pub mod ui;
pub mod unit_info_bar;
#[cfg(target_arch = "wasm32")]
pub mod wasm_keyboard;

use bevy::prelude::*;

/// Game state enum — shared across the render view.
/// Paused is a boolean resource, not a state variant.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, States, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

/// Controls what happens when entering Playing state.
#[derive(Resource, Default)]
pub enum NeedsGameReset {
    /// Pause recovery — no reset
    #[default]
    None,
    /// Restart/replay with current map size
    SameSize,
    /// New game with specified map size
    NewGame(simulation::map::MapSize),
    /// Load a replay file
    Replay(simulation::replay::ReplayFile),
}

/// Whether to auto-record replays. Defaults to true.
#[derive(Resource)]
pub struct AutoRecordReplay(pub bool);

impl Default for AutoRecordReplay {
    fn default() -> Self {
        Self(true)
    }
}

pub struct RenderViewPlugin;

impl Plugin for RenderViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<NeedsGameReset>()
            .init_resource::<AutoRecordReplay>()
            .init_resource::<crate::selection::SelectionState>()
            .add_plugins(crate::ui::UiPlugin)
            .add_systems(Startup, crate::camera::setup_camera)
            .init_resource::<crate::unit_info_bar::UnitInfoBarSettings>();

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Startup, crate::wasm_keyboard::setup_wasm_keyboard);
        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, crate::wasm_keyboard::maintain_wasm_keyboard_focus);
        #[cfg(target_arch = "wasm32")]
        app.add_systems(Last, crate::wasm_keyboard::clear_wasm_keyboard_just_pressed);

        app
            // Lifecycle: reset on enter Playing, cleanup on exit
            .add_systems(
                OnEnter(GameState::Playing),
                reset_game_system.before(crate::ui::hud::setup_hud),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                crate::ui::hud::setup_hud.after(reset_game_system),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_playing_system)
            // Visual systems: always run during Playing (including replay)
            .add_systems(
                Update,
                (
                    crate::debug_shape::draw_debug_shapes_system,
                    crate::debug_shape::draw_dropped_shields_system,
                    crate::debug_shape::draw_boundary_walls_system,
                    crate::unit_info_bar::unit_info_bar_system,
                    crate::unit_info_bar::info_bar_mode_toggle_system,
                    crate::selection::selection_visual_system,
                    crate::selection::drag_visual_system,
                    crate::selection::waypoint_cleanup_system,
                    check_victory_system,
                )
                    .run_if(
                        in_state(GameState::Playing)
                            .and_then(not(resource_exists_and_equals(bevy_adapter::Paused(true))))
                            .and_then(not(replay_seeking)),
                    ),
            )
            // Input systems: only when Playing AND Live (not replay)
            .add_systems(
                Update,
                (
                    crate::selection::selection_click_system,
                    crate::selection::drag_select_system,
                    crate::selection::selection_shortcut_system,
                    crate::selection::command_issue_system,
                    crate::selection::seek_stance_shortcut_system,
                )
                    .run_if(
                        in_state(GameState::Playing)
                            .and_then(not(resource_exists_and_equals(bevy_adapter::Paused(true))))
                            .and_then(not(replay_seeking))
                            .and_then(not(resource_exists_and_equals(bevy_adapter::GameMode::Replay))),
                    ),
            )
            // Camera: always active
            .add_systems(
                Update,
                (
                    crate::camera::camera_drag_system,
                    crate::camera::camera_edge_scroll_system,
                    crate::camera::camera_zoom_system,
                    crate::camera::center_on_player_city,
                ),
            );
    }
}

/// Run condition: true when replay is actively seeking (skip rendering).
fn replay_seeking(status: Option<Res<bevy_adapter::replay::ReplayStatus>>) -> bool {
    status.is_some_and(|s| s.is_seeking)
}

/// Check if all cities of one faction are gone.
fn check_victory_system(
    mut sim_world: bevy::ecs::system::NonSendMut<bevy_adapter::tick::SimulationWorld>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let world = &mut sim_world.0;
    let mut query = world.query::<(&simulation::soldier::FactionComponent,)>();
    let mut player = false;
    let mut enemy = false;
    for (f,) in query.iter(world) {
        match f.0 {
            simulation::types::Faction::Player => player = true,
            simulation::types::Faction::Enemy => enemy = true,
            _ => {}
        }
    }
    if !player || !enemy {
        next_state.set(GameState::GameOver);
    }
}

/// Reset game state when entering Playing.
/// If NeedsGameReset is true, fully resets the simulation world.
/// Always clears the paused flag.
#[allow(clippy::too_many_arguments)]
fn reset_game_system(
    mut commands: Commands,
    mut sim_world: bevy::ecs::system::NonSendMut<bevy_adapter::tick::SimulationWorld>,
    mut mapper: ResMut<bevy_adapter::mapper::UnitIdMapper>,
    mut tick_clock: ResMut<bevy_adapter::tick::TickClock>,
    mut cmd_buf: ResMut<simulation::command::CommandBuffer>,
    mut pending: ResMut<bevy_adapter::tick::PendingEvents>,
    mut selection: ResMut<crate::selection::SelectionState>,
    mut needs_reset: ResMut<NeedsGameReset>,
    mut paused: ResMut<bevy_adapter::Paused>,
    mut game_active: ResMut<bevy_adapter::GameActive>,
    mut _driver: ResMut<bevy_adapter::driver::SimulationDriver>,
    mut current_map_size: ResMut<bevy_adapter::CurrentMapSize>,
    mut recorder: ResMut<bevy_adapter::replay::ReplayRecorder>,
    auto_record: Res<AutoRecordReplay>,
    map_bounds: Option<ResMut<bevy_adapter::MapBounds>>,
    game_entities: Query<Entity, With<bevy_adapter::binding::LogicEntityRef>>,
) {
    paused.0 = false;
    game_active.0 = true;

    let _is_replay = matches!(&*needs_reset, NeedsGameReset::Replay(_));

    let (map_size, replay_file) = match std::mem::replace(&mut *needs_reset, NeedsGameReset::None) {
        NeedsGameReset::None => (None, None),
        NeedsGameReset::SameSize => (Some(current_map_size.0), None),
        NeedsGameReset::NewGame(size) => (Some(size), None),
        NeedsGameReset::Replay(replay) => (Some(replay.map_size), Some(replay)),
    };

    if let Some(map_size) = map_size {
        // Despawn all stale game entities
        for e in game_entities.iter() {
            commands.entity(e).despawn();
        }

        // Clear all game state
        mapper.clear();
        *tick_clock = bevy_adapter::tick::TickClock::default();
        cmd_buf.0.clear();
        pending.events.clear();
        selection.clear();

        // Rebuild simulation world
        let seed = replay_file.as_ref().map(|r| r.seed).unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        let mut world = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world, map_size);
        sim_world.0 = world;

        // Update current map size and MapBounds
        current_map_size.0 = map_size;

        // Initialize replay recorder (disabled when loading a replay)
        recorder.seed = seed;
        recorder.map_size = map_size;
        recorder.command_log.clear();
        recorder.is_recording = replay_file.is_none() && auto_record.0;

        let config = map_size.load_config();
        let w = config.width as f32;
        let h = config.height as f32;
        let new_bounds = bevy_adapter::MapBounds {
            width: w,
            height: h,
            wall_min_x: -w / 2.0,
            wall_min_y: -h / 2.0,
            wall_max_x: w * 1.5,
            wall_max_y: h * 1.5,
        };
        if let Some(mut bounds) = map_bounds {
            *bounds = new_bounds;
        } else {
            commands.insert_resource(new_bounds);
        }

        // Backfill: create Bevy entities for all simulation entities
        {
            use bevy_adapter::binding::{InterpolationData, LogicEntityRef, PresentationPosition};
            use simulation::soldier::{LogicalPosition, UnitIdComponent};
            let world = &mut sim_world.0;
            let to_spawn: Vec<(simulation::types::UnitId, Vec2)> = {
                let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
                query
                    .iter(world)
                    .map(|(_, id, pos)| {
                        (
                            id.0,
                            bevy::math::Vec2::new(pos.0.x.to_float(), pos.0.y.to_float()),
                        )
                    })
                    .collect()
            };
            for (unit_id, float_pos) in to_spawn {
                let entity = commands
                    .spawn((
                        LogicEntityRef(unit_id),
                        PresentationPosition(float_pos),
                        InterpolationData {
                            previous_logical_pos: float_pos,
                            current_logical_pos: float_pos,
                            is_new: true,
                        },
                    ))
                    .id();
                mapper.register(unit_id, entity);
            }
        }

        // If loading a replay, set up replay mode after entity backfill
        if let Some(replay) = replay_file {
            let total = replay.total_ticks;
            commands.insert_resource(bevy_adapter::driver::SimulationDriver::new_replay(replay));
            commands.insert_resource(bevy_adapter::GameMode::Replay);
            commands.insert_resource(bevy_adapter::replay::ReplayStatus {
                is_replay: true,
                total_ticks: total,
                is_seeking: false,
            });
        }
    }
}

/// Cleanup when leaving Playing state (to MainMenu or GameOver).
#[allow(clippy::too_many_arguments)]
fn cleanup_playing_system(
    mut commands: Commands,
    mut game_active: ResMut<bevy_adapter::GameActive>,
    mut driver: ResMut<bevy_adapter::driver::SimulationDriver>,
    mut status: ResMut<bevy_adapter::replay::ReplayStatus>,
    mut recorder: ResMut<bevy_adapter::replay::ReplayRecorder>,
    tick_clock: Res<bevy_adapter::tick::TickClock>,
    hud_query: Query<Entity, With<crate::ui::hud::HudRoot>>,
    pause_query: Query<Entity, With<crate::ui::pause::PauseUI>>,
) {
    game_active.0 = false;
    *driver = bevy_adapter::driver::SimulationDriver::new_live();
    *status = bevy_adapter::replay::ReplayStatus::default();
    commands.insert_resource(bevy_adapter::GameMode::Live);

    // Save replay file if recording was active
    if recorder.is_recording && !recorder.command_log.is_empty() {
        let replay = recorder.finish(tick_clock.current_tick);
        let ron = replay.to_ron();
        let dir = std::path::PathBuf::from("replays");
        let _ = std::fs::create_dir_all(&dir);
        let filename = format!("replay_{}.ron", chrono_timestamp());
        let path = dir.join(&filename);
        if let Err(e) = std::fs::write(&path, &ron) {
            bevy::log::warn!("Failed to save replay: {}", e);
        } else {
            bevy::log::info!("Replay saved: {}", path.display());
        }
    }
    recorder.is_recording = false;

    for e in hud_query.iter() {
        commands.entity(e).despawn();
    }
    for e in pause_query.iter() {
        commands.entity(e).despawn();
    }
}

/// Simple timestamp for filenames (no external crate needed).
fn chrono_timestamp() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

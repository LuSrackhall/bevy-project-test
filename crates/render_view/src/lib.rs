pub mod debug_shape;
pub mod camera;
pub mod selection;
pub mod ui;
pub mod unit_info_bar;

use bevy::prelude::*;
use bevy::ui_widgets::{ButtonPlugin, MenuPlugin};
use bevy::ui_widgets::popover::PopoverPlugin;
use bevy::input_focus::InputDispatchPlugin;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;

/// Game state enum — shared across the render view.
/// Paused is a boolean resource, not a state variant.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, States, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

/// When true, entering Playing will reset the simulation world.
#[derive(Resource, Default)]
pub struct NeedsGameReset(pub bool);

pub struct RenderViewPlugin;

impl Plugin for RenderViewPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .init_resource::<NeedsGameReset>()
            .init_resource::<crate::selection::SelectionState>()
            .add_plugins((ButtonPlugin, MenuPlugin, PopoverPlugin, InputDispatchPlugin, TabNavigationPlugin))
            .add_plugins(crate::ui::UiPlugin)
            .add_systems(Startup, crate::camera::setup_camera)
            .init_resource::<crate::unit_info_bar::UnitInfoBarSettings>()
            // Lifecycle: reset on enter Playing, cleanup on exit
            .add_systems(OnEnter(GameState::Playing), reset_game_system.before(crate::ui::hud::setup_hud))
            .add_systems(OnEnter(GameState::Playing), crate::ui::hud::setup_hud.after(reset_game_system))
            .add_systems(OnExit(GameState::Playing), cleanup_playing_system)
            // Gameplay systems: only when Playing AND not paused
            .add_systems(Update, (
                crate::debug_shape::draw_debug_shapes_system,
                crate::debug_shape::draw_dropped_shields_system,
                crate::unit_info_bar::unit_info_bar_system,
                crate::unit_info_bar::info_bar_mode_toggle_system,
                crate::selection::selection_click_system,
                crate::selection::drag_select_system,
                crate::selection::selection_shortcut_system,
                crate::selection::selection_visual_system,
                crate::selection::drag_visual_system,
                crate::selection::command_issue_system,
                crate::selection::seek_stance_shortcut_system,
                crate::selection::waypoint_cleanup_system,
                check_victory_system,
            ).run_if(in_state(GameState::Playing).and(not(resource_exists_and_equals(bevy_adapter::Paused(true))))))
            // Camera: always active
            .add_systems(Update, (
                crate::camera::camera_drag_system,
                crate::camera::camera_zoom_system,
                crate::camera::center_on_player_city,
            ));
    }
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
    game_entities: Query<Entity, With<bevy_adapter::binding::LogicEntityRef>>,
) {
    paused.0 = false;
    game_active.0 = true;

    if needs_reset.0 {
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

        // Rebuild simulation world with random seed
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut world = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world);
        sim_world.0 = world;

        needs_reset.0 = false;

        // Backfill: create Bevy entities for all simulation entities
        {
            use bevy_adapter::binding::{LogicEntityRef, PresentationPosition, InterpolationData};
            use simulation::soldier::{UnitIdComponent, LogicalPosition};
            let world = &mut sim_world.0;
            let to_spawn: Vec<(simulation::types::UnitId, Vec2)> = {
                let mut query = world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
                query.iter(world)
                    .map(|(_, id, pos)| (id.0, bevy::math::Vec2::new(pos.0.x.to_float(), pos.0.y.to_float())))
                    .collect()
            };
            for (unit_id, float_pos) in to_spawn {
                let entity = commands.spawn((
                    LogicEntityRef(unit_id),
                    PresentationPosition(float_pos),
                    InterpolationData {
                        previous_logical_pos: float_pos,
                        current_logical_pos: float_pos,
                        is_new: true,
                    },
                )).id();
                mapper.register(unit_id, entity);
            }
        }
    }
}

/// Cleanup when leaving Playing state (to MainMenu or GameOver).
fn cleanup_playing_system(
    mut commands: Commands,
    mut game_active: ResMut<bevy_adapter::GameActive>,
    hud_query: Query<Entity, With<crate::ui::hud::HudRoot>>,
    pause_query: Query<Entity, With<crate::ui::pause::PauseUI>>,
) {
    game_active.0 = false;
    for e in hud_query.iter() {
        commands.entity(e).despawn();
    }
    for e in pause_query.iter() {
        commands.entity(e).despawn();
    }
}

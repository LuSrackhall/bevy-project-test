use bevy::prelude::*;
use std::path::PathBuf;

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
    /// 局域网大厅 — 房间列表 + 创建/加入
    LanLobby,
    /// 等待房间 — 网络模式：TCP 已连接但 GameStarted 尚未收到
    Lobby,
    Playing,
    GameOver,
}

/// Read the local human player's id from the simulation world.
/// Returns 0 (Player faction) if not set (single-player default).
pub(crate) fn local_player_id(sim: &bevy_adapter::tick::SimulationWorld) -> u8 {
    sim.world_ref()
        .get_resource::<simulation::types::LocalPlayerId>()
        .map(|r| r.0)
        .unwrap_or(0)
}


/// Controls what happens when entering Playing state.
#[derive(Resource, Default)]
pub enum NeedsGameReset {
    #[default]
    None,
    SameSize,
    NewGame(simulation::map::MapSize),
    Replay(simulation::replay::ReplayFile),
    Network { relay_addr: String, player_count: u8, player_id: Option<u8>, relay_id: bevy_adapter::discovery::RelayId },
}

/// Whether the local client created the room (is the host).
#[derive(Resource, Default)]
pub struct IsHost(pub bool);

/// Player list displayed in the lobby waiting room.
#[derive(Resource, Default)]
pub struct LobbyPlayerList(pub Vec<bevy_adapter::network::LobbyPlayerState>);

/// Lobby 连接阶段
#[derive(Debug, Clone)]
pub enum LobbyPhase {
    Connecting,
    Connected,
    Ready,
    Failed(String),
}

impl Default for LobbyPhase {
    fn default() -> Self { Self::Connecting }
}

/// Lobby 连接状态（由 lobby_update_system 驱动）
#[derive(Resource, Default)]
pub struct LobbyConnectionState {
    pub phase: LobbyPhase,
}

/// 跨线程 TCP 连接状态轮询器（内部包裹 Arc<Mutex<Option<Result>>>）
#[derive(Resource)]
pub struct ConnectionPollRx(pub std::sync::Arc<std::sync::Mutex<Option<Result<(), String>>>>);

impl Default for ConnectionPollRx {
    fn default() -> Self { Self(std::sync::Arc::new(std::sync::Mutex::new(None))) }
}

/// Whether to auto-record replays. Defaults to true.
#[derive(Resource)]
pub struct AutoRecordReplay(pub bool);

impl Default for AutoRecordReplay {
    fn default() -> Self {
        Self(true)
    }
}

/// Received from relay via GameStarted message — seed for deterministic world creation.
#[derive(Resource, Default)]
pub struct NetworkGameStart {
    pub seed: u64,
    pub player_id: u8,
    pub player_count: u8,

    pub received: bool,
}

pub struct RenderViewPlugin;

impl Plugin for RenderViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<NeedsGameReset>()
            .init_resource::<AutoRecordReplay>()
            .init_resource::<NetworkGameStart>()
            .init_resource::<crate::selection::SelectionState>()
            .init_resource::<CreateRoomRequest>()
            .init_resource::<JoinRoomRequest>()
            .init_resource::<LocalPlayerIdentity>()
            .init_resource::<LobbyPlayerList>()
            .init_resource::<IsHost>()
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
            // Lobby: entry + non-blocking connect + async polling
            .add_systems(OnEnter(GameState::Lobby), setup_lobby_system)
            .add_systems(
                Update,
                lobby_update_system.run_if(in_state(GameState::Lobby)),
            )
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
                    crate::debug_shape::draw_dropped_shields_system,
                    crate::debug_shape::draw_boundary_walls_system,
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
                    )
                    .before(bevy_adapter::SimulationTickSet)
                    // Commands must be created BEFORE network_flush_system drains cmd_buf,
                    // otherwise a click is flushed a frame late to an already-finalized tick.
                    .before(bevy_adapter::transport::network_flush_system),
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
            )
            // LAN Room Creation: Request Resource + Integration System
            .init_resource::<CreateRoomRequest>()
            .add_systems(
                Update,
                handle_create_room.run_if(in_state(GameState::LanLobby)),
            )
            .add_systems(Update,
                handle_join_room.run_if(in_state(GameState::LanLobby)),
            );

        // Debug-only visual systems (gated behind debug_render feature per constitution §21)
        #[cfg(feature = "debug_render")]
        app.add_systems(
            Update,
            (
                crate::debug_shape::draw_debug_shapes_system,
                crate::unit_info_bar::unit_info_bar_system,
            )
                .run_if(
                    in_state(GameState::Playing)
                        .and_then(not(resource_exists_and_equals(bevy_adapter::Paused(true))))
                        .and_then(not(replay_seeking)),
                ),
        );
    }
}

/// Run condition: true when replay is actively seeking (skip rendering).
fn replay_seeking(status: Option<Res<bevy_adapter::replay::ReplayStatus>>) -> bool {
    status.is_some_and(|s| s.is_seeking)
}

/// Check if any active player faction has been eliminated.
fn check_victory_system(
    sim_world: bevy::ecs::system::NonSend<bevy_adapter::tick::SimulationWorld>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let lid = crate::local_player_id(&*sim_world);
    let world = sim_world.world_ref();

    // Collect active player factions from PlayerSlots (ignoring neutrals like FactionId(2))
    use simulation::types::PlayerSlots;
    let active_factions: Vec<simulation::types::FactionId> = world
        .get_resource::<PlayerSlots>()
        .map(|s| {
            s.slots
                .iter()
                .filter(|s| s.controller.is_active())
                .map(|s| s.faction)
                .collect()
        })
        .unwrap_or_default();

    let mut has_my_faction = false;
    let mut has_enemy = false;
    let mut q = sim_world.query::<(&simulation::soldier::FactionComponent,)>();
    for (f,) in q.iter(world) {
        if f.0 == simulation::types::FactionId(lid) {
            has_my_faction = true;
        } else if active_factions.contains(&f.0) && f.0 != simulation::types::FactionId(lid) {
            has_enemy = true;
        }
    }
    if !has_my_faction || !has_enemy {
        next_state.set(GameState::GameOver);
    }
}


/// 进入 Lobby 状态：发起非阻塞 TCP 连接，插入传输资源。
fn setup_lobby_system(
    mut commands: Commands,
    mut _driver: ResMut<bevy_adapter::driver::SimulationDriver>,
    mut recorder: ResMut<bevy_adapter::replay::ReplayRecorder>,
    mut cmd_buf: ResMut<simulation::command::CommandBuffer>,
    mut network_start: ResMut<NetworkGameStart>,
    needs_reset: Res<NeedsGameReset>,
) {
    let network_config = match &*needs_reset {
        NeedsGameReset::Network { relay_addr, player_count, player_id, relay_id } => {
            Some((relay_addr.clone(), *player_count, *player_id, *relay_id))
        }
        _ => None,
    };
    let Some((relay_addr, player_count, player_id, relay_id)) = network_config else {
        bevy::log::error!("[LOBBY] NeedsGameReset is not Network");
        commands.insert_resource(NextState::<GameState>::default());
        return;
    };

    eprintln!("[SETUP_LOBBY] relay_addr={}, player={:?}, count={}", relay_addr, player_id, player_count);

    network_start.player_id = player_id.unwrap_or(0);
    network_start.player_count = player_count;

    let effective_id = player_id.unwrap_or(0);
    bevy::log::info!("[LOBBY] Initializing network (relay={}, player={}/{})", relay_addr, effective_id, player_count);

    use bevy_adapter::network::NetworkEventReceiver;
    use bevy_adapter::transport::spawn_network_client_nonblocking;
    let event_receiver = NetworkEventReceiver::default();
    let (receiver, sender, handle, status) = spawn_network_client_nonblocking(
        relay_addr.clone(), 1, effective_id, 1, event_receiver.clone(), relay_id,
    );

    // Insert transport resources immediately (tokio thread runs in background)
    commands.insert_resource(event_receiver);
    commands.insert_resource(receiver);
    commands.insert_resource(sender);
    commands.insert_resource(handle);
    commands.insert_resource(ConnectionPollRx(status.inner_arc()));
    commands.insert_resource(LobbyConnectionState { phase: LobbyPhase::Connecting });

    // Reset bootstrap phase to Init so wire() works after TCP connects
    _driver.bootstrap_phase = bevy_adapter::session::bootstrap::BootstrapPhase::Init;

    bevy::log::info!("[LOBBY] Connection initiated — polling TCP status...");
}

/// 轮询 TCP 连接状态 + 完成 bootstrap + 等待 GameStarted
pub fn lobby_update_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut network_start: ResMut<NetworkGameStart>,
    poll_rx: Option<Res<ConnectionPollRx>>,
    mut lobby_state: Option<ResMut<LobbyConnectionState>>,
    event_receiver: Option<Res<bevy_adapter::network::NetworkEventReceiver>>,
    mut _driver: Option<ResMut<bevy_adapter::driver::SimulationDriver>>,
    mut network_active: Option<ResMut<bevy_adapter::NetworkActive>>,
) {
    let Some(mut state) = lobby_state else { return };

    match state.phase.clone() {
        LobbyPhase::Connecting => {
            // Poll TCP connection status
            if let Some(ref rx) = poll_rx {
                let conn_result = rx.0.lock().unwrap().take();
                match conn_result {
                    Some(Ok(())) => {
                        bevy::log::info!("[LOBBY] TCP connected — completing bootstrap...");
                        // Manually wire: set up NetworkCommandSource
                        use bevy_adapter::driver::CommandSource;
                        use bevy_adapter::network::NetworkCommandSource;
                        use bevy_adapter::session::bootstrap::BootstrapPhase;
                        if let Some(ref mut d) = _driver {
                            d.source = CommandSource::Network(
                                NetworkCommandSource::new(1, network_start.player_id, 3),
                            );
                            d.bootstrap_phase = BootstrapPhase::Wired;
                        }
                        // Enable network systems (poll, flush)
                        if let Some(ref mut na) = network_active {
                            na.0 = true;
                        }
                        state.phase = LobbyPhase::Connected;
                    }
                    Some(Err(e)) => {
                        bevy::log::error!("[LOBBY] TCP connect failed: {}", e);
                        state.phase = LobbyPhase::Failed(e);
                    }
                    None => {} // Still connecting
                }
            }
        }
        LobbyPhase::Connected => {
            use bevy_adapter::network::NetworkEvent;
            use bevy_adapter::driver::CommandSource;
            let Some(receiver) = event_receiver else { return };
            let events = receiver.drain_all();
            for event in &events {
                if let NetworkEvent::GameJoined { player_id, player_count } = event {
                    bevy::log::info!("[LOBBY] Identity assigned: player={}/{}", player_id, player_count);
                    // Update NetworkCommandSource with relay-assigned player_id
                    if let Some(ref mut d) = _driver {
                        if let CommandSource::Network(ref mut ns) = d.source {
                            ns.player_id = *player_id;
                        }
                    }
                    // Update LocalPlayerIdentity
                    let mut identity = crate::LocalPlayerIdentity::default();
                    identity.player_id = *player_id;
                    identity.player_count = *player_count;
                    identity.assigned = true;
                    commands.insert_resource(identity);
                    // Also update NetworkGameStart so reset_game_system uses the
                    // relay-assigned player_id (not the temporary 0 from setup_lobby_system)
                    network_start.player_id = *player_id;
                    network_start.player_count = *player_count;
                }
                if let NetworkEvent::GameStarted { game_id: _, seed, .. } = event {
                    bevy::log::info!("[LOBBY] GameStarted received! seed={}", seed);
                    network_start.seed = *seed;
                    network_start.received = true;
                    next_state.set(GameState::Playing);
                    return;
                }
                if let NetworkEvent::LobbyUpdate { players, .. } = event {
                    // Store player list for UI rendering (C2)
                    commands.insert_resource(LobbyPlayerList(players.clone()));
                    // Only set Ready if the local player is ready
                    let local_ready = players.iter().any(|p| {
                        p.player_id == network_start.player_id && p.ready
                    });
                    if local_ready {
                        state.phase = LobbyPhase::Ready;
                        return;
                    }
                }
            }
        }
        LobbyPhase::Ready => {
            use bevy_adapter::network::NetworkEvent;
            let Some(receiver) = event_receiver else { return };
            let events = receiver.drain_all();
            for event in &events {
                if let NetworkEvent::GameStarted { game_id: _, seed, .. } = event {
                    bevy::log::info!("[LOBBY] GameStarted received (from Ready)! seed={}", seed);
                    network_start.seed = *seed;
                    network_start.received = true;
                    next_state.set(GameState::Playing);
                    return;
                }
            }
        }
        LobbyPhase::Failed(_) => {} // Handled by UI cancel button
    }
}

/// 退出 Lobby 时清理网络资源（非取消路径的兜底清理）。
fn cleanup_lobby_network(
    mut commands: Commands,
    mut network_active: ResMut<bevy_adapter::NetworkActive>,
) {
    network_active.0 = false;
    commands.remove_resource::<ConnectionPollRx>();
    commands.remove_resource::<LobbyConnectionState>();
}

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
    mut needs_reset: ResMut<NeedsGameReset>,
    mut paused: ResMut<bevy_adapter::Paused>,
    mut game_active: ResMut<bevy_adapter::GameActive>,
    mut network_active: ResMut<bevy_adapter::NetworkActive>,
    mut driver: ResMut<bevy_adapter::driver::SimulationDriver>,
    mut current_map_size: ResMut<bevy_adapter::CurrentMapSize>,
    mut recorder: ResMut<bevy_adapter::replay::ReplayRecorder>,
    mut network_start: ResMut<NetworkGameStart>,
    game_entities: Query<Entity, With<bevy_adapter::binding::LogicEntityRef>>,
) {
    paused.0 = false;
    game_active.0 = true;
    network_active.0 = false; // Network mode already active, disable lobby

    let (map_size, replay_file, network_config) = match std::mem::replace(&mut *needs_reset, NeedsGameReset::None) {
        NeedsGameReset::None => (None, None, None),
        NeedsGameReset::SameSize => (Some(current_map_size.0), None, None),
        NeedsGameReset::NewGame(size) => (Some(size), None, None),
        NeedsGameReset::Replay(replay) => (Some(replay.map_size), Some(replay), None),
        // 网络对局默认地图 Medium。重连(场景 A)不重建世界,地图保持当前对局地图;
        // 场景 B 重建时由调用方以对局 map_size 调 generate_map(R4 保证一致)。
        NeedsGameReset::Network { .. } => (Some(simulation::map::MapSize::Medium), None, Some(()))
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
        commands.init_resource::<bevy_adapter::tick::PendingEvents>();
        commands.init_resource::<crate::selection::SelectionState>();

        // Rebuild simulation world
        let seed = if network_config.is_some() && network_start.received {
            // Network mode: use seed broadcast by relay (deterministic across clients)
            network_start.seed
        } else {
            replay_file.as_ref().map(|r| r.seed).unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            })
        };
        let mut world = if network_start.received {
            // R5: 世界重建封装在 bevy_adapter 会话层(render_view 不直触仿真)
            bevy_adapter::session::reconnect::rebuild_world(
                seed, network_start.player_count, network_start.player_id
            )
        } else {
            simulation::init_simulation_world(seed)
        };
        simulation::map::generate_map(&mut world, map_size);
        // For network mode, store the local player's id so input/selection systems
        // can filter units and issue commands with the correct player_id.
        if network_start.received {
            world.insert_resource(simulation::types::LocalPlayerId(network_start.player_id));
        }
        sim_world.set_world(world);

        // Update current map size and MapBounds
        current_map_size.0 = map_size;

        // Initialize replay recorder (disabled when loading a replay)
        recorder.seed = seed;
        recorder.map_size = map_size;
        recorder.command_log.clear();
        recorder.is_recording = replay_file.is_none();

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
        commands.insert_resource(new_bounds);

        // Backfill: create Bevy entities for all simulation entities
        {
            use bevy_adapter::binding::{InterpolationData, LogicEntityRef, PresentationPosition};
            use simulation::soldier::{LogicalPosition, UnitIdComponent};
            let mut query = sim_world.query::<(Entity, &UnitIdComponent, &LogicalPosition)>();
            let world = sim_world.world_ref();
            let to_spawn: Vec<(simulation::types::UnitId, Vec2)> = {
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

        // If starting a network game, finalize bootstrap and activate the driver.
        // bootstrap_session was already called in setup_lobby_system.
        if network_config.is_some() {
            use bevy_adapter::session::bootstrap::BootstrapPhase;
            driver.bootstrap_phase = BootstrapPhase::Active;
            commands.insert_resource(bevy_adapter::GameMode::Live);
            // Clear the lobby resource to free memory
            network_start.received = false;
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
    mut network_active: ResMut<bevy_adapter::NetworkActive>,
    tick_clock: Res<bevy_adapter::tick::TickClock>,
    hud_query: Query<Entity, With<crate::ui::hud::HudRoot>>,
    pause_query: Query<Entity, With<crate::ui::pause::PauseUI>>,
) {
    game_active.0 = false;
    network_active.0 = false;
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

    // Clean up network resources (stops the tokio thread via Drop)
    commands.remove_resource::<bevy_adapter::transport::NetworkClientHandle>();
    commands.remove_resource::<bevy_adapter::network::NetworkEventReceiver>();
    commands.remove_resource::<bevy_adapter::transport::NetworkSender>();
    commands.remove_resource::<bevy_adapter::transport::NetworkReceiver>();
}

/// Simple timestamp for filenames (no external crate needed).
fn chrono_timestamp() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

// ═══════════════════════════════════════════════════════════════
// LAN Room Creation — Intent Resource + Integration System
// ═══════════════════════════════════════════════════════════════

/// Request to create a LAN room. Set by UI, consumed by the integration system.
#[derive(Resource, Default)]
pub struct CreateRoomRequest {
    pub requested: bool,
    pub room_name: String,
    pub map_id: String,
    pub max_players: u8,
}

/// Request to join a LAN room. Set by UI, consumed by Join Integration System.
#[derive(Resource)]
pub struct JoinRoomRequest {
    pub requested: bool,
    pub room_id: bevy_adapter::discovery::RoomId,
    pub relay_id: bevy_adapter::discovery::RelayId,
    pub endpoint: String,
    pub max_players: u8,
}

impl Default for JoinRoomRequest {
    fn default() -> Self {
        Self {
            requested: false,
            room_id: bevy_adapter::discovery::RoomId(0),
            relay_id: bevy_adapter::discovery::RelayId(0),
            endpoint: String::new(),
            max_players: 2,
        }
    }
}

/// Relay-authoritative player identity. Written only on GameJoined.
#[derive(Resource, Default)]
pub struct LocalPlayerIdentity {
    pub player_id: u8,
    pub player_count: u8,
    pub assigned: bool,
}

/// Integration system: reads CreateRoomRequest and calls SessionController.
fn handle_create_room(
    mut request: ResMut<CreateRoomRequest>,
    mut controller: ResMut<bevy_adapter::session_host::SessionController>,
    mut needs_reset: ResMut<NeedsGameReset>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if !request.requested {
        return;
    }
    request.requested = false; // Consume the request

    use bevy_adapter::discovery::{RoomId, RoomMetadata, RoomState};
    let room = RoomMetadata {
        room_id: RoomId(0),
        room_name: if request.room_name.is_empty() {
            format!("房间_{}", chrono_timestamp().chars().take(6).collect::<String>())
        } else {
            request.room_name.clone()
        },
        map_id: request.map_id.clone(),
        current_players: 1,
        max_players: request.max_players,
        state: RoomState::Waiting,
    };
    match controller.create_session(room) {
        Ok(_) => {
            bevy::log::info!("[LAN] Room created successfully");
            // Transition host into Lobby by connecting to the local relay
            if let Some(session) = controller.current_session() {
                let endpoint = session.relay.endpoint();
                let relay_id = session.relay.relay_id();
                *needs_reset = NeedsGameReset::Network {
                    relay_addr: format!("127.0.0.1:{}", endpoint.port()),
                    player_count: request.max_players,
                    player_id: Some(0), // Host is player 0
                    relay_id,
                };
                commands.insert_resource(IsHost(true));
                next_state.set(GameState::Lobby);
            }
        }
        Err(e) => {
            bevy::log::error!("[LAN] Failed to create room: {}", e);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// LAN Join Flow — JoinRoomRequest + Integration System
// ═══════════════════════════════════════════════════════════════

/// Integration system: reads JoinRoomRequest and triggers TCP connection + Lobby transition.
fn handle_join_room(
    mut request: ResMut<JoinRoomRequest>,
    mut needs_reset: ResMut<NeedsGameReset>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if !request.requested {
        return;
    }
    request.requested = false;
    eprintln!("[HANDLE_JOIN] called — endpoint={}, relay_id={:?}", request.endpoint, request.relay_id);

    let max_players = request.max_players;
    *needs_reset = NeedsGameReset::Network {
        relay_addr: request.endpoint.clone(),
        player_count: max_players,
        player_id: None, // Relay assigns player_id
        relay_id: request.relay_id,
    };
    commands.insert_resource(IsHost(false));
    next_state.set(GameState::Lobby);

    bevy::log::info!(
        "[LAN] Joining room (room_id={:?}, endpoint={})",
        request.room_id,
        request.endpoint,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_join_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_resource::<JoinRoomRequest>();
        app.init_resource::<NeedsGameReset>();
        app.init_resource::<IsHost>();
        app.add_systems(Update, handle_join_room.run_if(in_state(GameState::LanLobby)));
        app.insert_state(GameState::LanLobby);
        app
    }

    #[test]
    fn test_handle_join_room_ignores_if_not_requested() {
        let mut app = make_join_app();
        app.update();
        assert!(matches!(*app.world().resource::<NeedsGameReset>(), NeedsGameReset::None));
    }

    #[test]
    fn test_handle_join_room_sets_network_reset() {
        let mut app = make_join_app();
        {
            let mut req = app.world_mut().resource_mut::<JoinRoomRequest>();
            req.requested = true;
            req.endpoint = "192.168.1.157:55347".into();
            req.relay_id = bevy_adapter::discovery::RelayId(42);
        }
        app.update();
        match &*app.world().resource::<NeedsGameReset>() {
            NeedsGameReset::Network { relay_addr, player_count, player_id, relay_id } => {
                assert_eq!(relay_addr, "192.168.1.157:55347");
                assert_eq!(*player_count, 2);
                assert!(player_id.is_none());
                assert_eq!(*relay_id, bevy_adapter::discovery::RelayId(42));
            }
            _ => panic!("Expected NeedsGameReset::Network"),
        }
    }

    #[test]
    fn test_handle_join_room_sets_is_host_false() {
        let mut app = make_join_app();
        app.world_mut().resource_mut::<JoinRoomRequest>().requested = true;
        app.update();
        assert!(!app.world().resource::<IsHost>().0);
    }

    #[test]
    fn test_handle_join_room_consumes_request() {
        let mut app = make_join_app();
        app.world_mut().resource_mut::<JoinRoomRequest>().requested = true;
        app.update();
        assert!(!app.world().resource::<JoinRoomRequest>().requested);
    }
}

//! Regression test: the render_view lobby must not drop a GameStarted that
//! arrives in the SAME drained batch as a ready LobbyUpdate.
//!
//! Real flow: when the last player readies, the relay broadcasts LobbyUpdate
//! then GameStarted back-to-back on the Control channel. Both land in one
//! `NetworkEventReceiver.drain_all()` batch. The Connected-phase handler must
//! process BOTH — if it returns early on the ready LobbyUpdate, GameStarted is
//! consumed and lost, and the client is stuck in the lobby forever.

use bevy::prelude::*;
use bevy_adapter::driver::SimulationDriver;
use bevy_adapter::network::{LobbyPlayerState, NetworkEvent, NetworkEventReceiver};
use render_view::{GameState, LobbyConnectionState, LobbyPhase, NetworkGameStart, lobby_update_system};

/// Build a minimal app in the Lobby state with a Connected lobby session and a
/// player-1 identity (relay-assigned), then push a single batch:
/// `[LobbyUpdate(player 1 ready), GameStarted]`.
fn make_connected_lobby_app() -> App {
    let mut app = App::new();
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_resource::<NetworkGameStart>();
    app.init_resource::<render_view::ConnectionPollRx>();
    app.insert_resource(LobbyConnectionState { phase: LobbyPhase::Connected });
    app.init_resource::<NetworkEventReceiver>();
    app.init_resource::<bevy_adapter::NetworkActive>();
    app.insert_resource(SimulationDriver::new_network());
    app.insert_state(GameState::Lobby);
    app.add_systems(Update, lobby_update_system.run_if(in_state(GameState::Lobby)));

    app.world_mut().resource_mut::<NetworkGameStart>().player_id = 1;
    app
}

#[test]
fn test_game_started_same_batch_as_ready_lobby_update_is_not_dropped() {
    let mut app = make_connected_lobby_app();

    {
        let events = app.world_mut().resource_mut::<NetworkEventReceiver>();
        events.push(NetworkEvent::LobbyUpdate {
            game_id: 1,
            players: vec![
                LobbyPlayerState { player_id: 0, ready: true, selected_map: None },
                LobbyPlayerState { player_id: 1, ready: true, selected_map: None },
            ],
        });
        events.push(NetworkEvent::GameStarted {
            game_id: 1,
            seed: 42,
            player_count: 2,
            map_size: simulation::map::MapSize::Medium,
        });
    }

    app.update();

    let ns = app.world().resource::<NetworkGameStart>();
    assert!(
        ns.received,
        "GameStarted arriving in the same batch as a ready LobbyUpdate was dropped — \
         client is stuck in the lobby forever (无法开局)"
    );
    assert_eq!(ns.seed, 42);

    // The user-visible symptom is staying in the lobby: assert the state actually
    // transitions to Playing (NextState applied on the following frame).
    app.update();
    let state = app.world().resource::<bevy::state::state::State<GameState>>();
    assert_eq!(state.get(), &GameState::Playing, "lobby must transition to Playing");
}

#[test]
fn test_lobby_update_alone_transitions_to_ready_but_waits_for_game_started() {
    let mut app = make_connected_lobby_app();

    // Batch contains only LobbyUpdate (local player ready) — no GameStarted yet.
    app.world_mut()
        .resource_mut::<NetworkEventReceiver>()
        .push(NetworkEvent::LobbyUpdate {
            game_id: 1,
            players: vec![
                LobbyPlayerState { player_id: 0, ready: true, selected_map: None },
                LobbyPlayerState { player_id: 1, ready: true, selected_map: None },
            ],
        });

    app.update();

    // Should have transitioned to Ready phase and NOT set received.
    assert!(matches!(
        app.world().resource::<LobbyConnectionState>().phase,
        LobbyPhase::Ready
    ));
    assert!(!app.world().resource::<NetworkGameStart>().received);

    // A GameStarted in a LATER batch (next frame) must still start the game.
    app.world_mut()
        .resource_mut::<NetworkEventReceiver>()
        .push(NetworkEvent::GameStarted {
            game_id: 1,
            seed: 7,
            player_count: 2,
            map_size: simulation::map::MapSize::Medium,
        });
    app.update();

    assert!(
        app.world().resource::<NetworkGameStart>().received,
        "GameStarted in a later batch after Ready must still be consumed"
    );
    assert_eq!(app.world().resource::<NetworkGameStart>().seed, 7);
}

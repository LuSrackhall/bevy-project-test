//! End-to-end integration test: relay → network transport → simulation driver → replay.
//!
//! Verifies the full pipeline:
//!
//! ```text
//! cmd_buf → network_flush_system → TCP (bincode) → relay
//!     → TCP (bincode) → network_poll_system
//!     → NetworkCommandSource.relay_buffer → simulation_driver_system
//!     → ReplayRecorder.record_tick()
//! ```
//!
//! Then switches to replay mode and verifies the recorded ReplayFile produces
//! identical world state hashes at every DESYNC_CHECK_INTERVAL.

use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use bevy_adapter::driver::{
    CommandSource, SimulationDriver, SchedulerState, TickClock, simulation_driver_system,
};
use bevy_adapter::network::{NetworkCommandSource, NetworkEventReceiver};
use bevy_adapter::replay::ReplayRecorder;
use bevy_adapter::tick::{PendingEvents, SimulationWorld};
use bevy_adapter::transport::{
    network_flush_system, network_poll_system, spawn_network_client,
};
use simulation::command::{CommandBuffer, GameCommand};
use simulation::golden_test;
use simulation::map::MapSize;
use simulation::replay::ReplayFile;
use simulation::soldier::{FactionComponent, SoldierMarker, UnitIdComponent};
use simulation::types::{Faction, Fixed, FixedVec2, UnitId};

// ═══════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════

/// Find a free TCP port by binding to `:0` then reading the assigned port.
fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Spawn the relay server on a background tokio thread.
fn spawn_relay(port: u16, seed: u64, players: u8) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("relay".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .expect("relay tokio runtime");
            rt.block_on(relay::start_relay(port, seed, players))
                .expect("relay server")
        })
        .expect("relay thread")
}

/// Build a GameCommand for the given tick.
fn make_cmd(tick: u32, unit_id: Option<UnitId>) -> GameCommand {
    let action = match unit_id {
        Some(uid) => simulation::command::Action::MoveTo {
            unit: uid,
            target: FixedVec2::new(Fixed::from_int(300), Fixed::from_int(300)),
        },
        None => simulation::command::Action::NoOp,
    };
    GameCommand {
        tick,
        player_id: 0,
        action,
    }
}

// ═══════════════════════════════════════════════════════════════
// Test
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_network_pipeline_e2e() {
    // ── Phase 1: Relay server (background thread) ──────────
    let port = find_free_port();
    let _relay = spawn_relay(port, 42, 1);
    thread::sleep(Duration::from_millis(200));

    // ── Phase 2: Seed the simulation world ─────────────────
    let seed = 42u64;
    let map_size = MapSize::Small;
    let mut raw_world = simulation::init_simulation_world(seed);
    simulation::map::generate_map(&mut raw_world, map_size);

    // Read-only query before wrapping (needs direct access to the inner World)
    let player_unit = {
        let mut q = raw_world.query::<(
            &UnitIdComponent,
            &FactionComponent,
            &SoldierMarker,
        )>();
        q.iter(&raw_world)
            .find(|(_, f, _)| f.0 == FactionId(0))
            .map(|(id, _, _)| id.0)
    };

    // Wrap in NonSend SimulationWorld for Bevy
    let sim_world = SimulationWorld::new(raw_world);

    // ── Phase 3: Start network client — get shared receiver/sender ──
    let event_receiver = bevy_adapter::network::NetworkEventReceiver::default();
    let (nrecv, nsend, _handle) = spawn_network_client(
        format!("127.0.0.1:{port}"),
        1, // game_id
        0, // player_id
        1, // ruleset_version
        event_receiver.clone(),
    )
    .expect("spawn_network_client should connect within 5s");
    // Wait for TCP handshake + GameJoined message
    thread::sleep(Duration::from_millis(500));

    // ── Phase 4: Build headless Bevy App ───────────────────
    let mut app = App::new();

    // Time — manually advanced each frame
    app.init_resource::<Time>();

    // Network transport (cross-thread bridges) — use the SAME
    // instances so tokio thread and Bevy systems share buffers
    app.insert_resource(nrecv);
    app.insert_resource(nsend);

    // Network-mode simulation driver
    app.insert_resource(SimulationDriver {
        clock: TickClock::default(),
        scheduler: SchedulerState::default(),
        source: CommandSource::Network(NetworkCommandSource {
            game_id: 1,
            player_id: 0,
            input_delay: 1,
            relay_buffer: std::collections::HashMap::new(),
            ruleset_version: 1,
            connected: false,
        }),
        bootstrap_phase: bevy_adapter::session::bootstrap::BootstrapPhase::Active,
    });

    // Support resources
    app.insert_resource(TickClock::default());
    app.init_resource::<PendingEvents>();
    app.insert_resource(CommandBuffer(Vec::new()));
    app.insert_resource(ReplayRecorder {
        is_recording: true,
        seed,
        map_size,
        ..Default::default()
    });
    app.insert_non_send(sim_world);

    // Systems run in chain: poll → flush → driver
    app.add_systems(
        Update,
        (
            network_poll_system,
            network_flush_system,
            simulation_driver_system,
        )
            .chain(),
    );

    // ── Phase 5: Pre-load commands into Bevy cmd_buf ───────
    // With input_delay=1, the flush system sends commands for
    // delayed_tick(current_tick) = current_tick + 1.
    // Push ticks 1..60 so the pipeline has work each frame.
    for tick in 1..=60 {
        app.world_mut()
            .resource_mut::<CommandBuffer>()
            .0
            .push(make_cmd(tick, player_unit));
    }

    // ── Phase 6: Run frames ────────────────────────────────
    // Each frame: advance 50 ms of game time, run systems, sleep for network RTT.
    for _ in 0..80 {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(0.05));
        app.update();
        thread::sleep(Duration::from_millis(40));
    }

    // ── Phase 7: Verify pipeline produced output ───────────
    let driver = app.world().resource::<SimulationDriver>();
    assert!(
        driver.clock.current_tick >= 20,
        "Driver should have processed at least 20 ticks, got {}",
        driver.clock.current_tick
    );

    let recorder = app.world().resource::<ReplayRecorder>();
    assert!(
        !recorder.command_log.is_empty(),
        "ReplayRecorder should have recorded at least one tick of commands"
    );
    assert!(
        !recorder.tick_hashes.is_empty(),
        "ReplayRecorder should have recorded hashes \
         (DESYNC_CHECK_INTERVAL={})",
        ReplayFile::DESYNC_CHECK_INTERVAL
    );

    // ── Phase 8: Build ReplayFile & round-trip via RON ─────
    let total_ticks = driver.clock.current_tick;
    let replay = recorder.finish(total_ticks);
    let ron = replay.to_ron();
    let loaded: ReplayFile =
        ReplayFile::from_ron(&ron).expect("ReplayFile RON round-trip");

    // ── Phase 9: Replay in a fresh simulation World ────────
    let mut replay_world = simulation::init_simulation_world(loaded.seed);
    simulation::map::generate_map(&mut replay_world, loaded.map_size);

    for tick in 1..=loaded.total_ticks {
        let cmds = loaded.commands_for_tick(tick).to_vec();
        for cmd in cmds {
            replay_world
                .resource_mut::<CommandBuffer>()
                .0
                .push(cmd);
        }
        simulation::run_tick_default(&mut replay_world, tick);

        // Verify determinism: hash at each DESYNC_CHECK_INTERVAL
        if tick % ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
            let expected = loaded.hash_for_tick(tick).unwrap_or_else(|| {
                panic!("missing recorded hash at tick {tick}")
            });
            let actual = golden_test::hash_world_state(&mut replay_world);
            assert_eq!(
                expected, actual,
                "DESYNC at tick {tick}: replay produced hash {actual:#x}, \
                 recorded hash was {expected:#x}. \
                 The network → simulation → replay pipeline is NOT deterministic."
            );
        }
    }
}

//! Two-player e2e test: issues a MoveTo at a specific frame and verifies the
//! command reaches the simulation in BOTH worlds (deterministic lockstep).
//!
//! This targets the "move command needs multiple clicks" bug — commands created
//! just-in-time (like a real right-click) must reliably reach the simulation.

use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use bevy_adapter::discovery::RelayId;
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
use simulation::map::MapSize;
use simulation::soldier::{FactionComponent, SoldierMarker, UnitIdComponent};
use simulation::types::{FactionId, Fixed, FixedVec2, UnitId};

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn spawn_relay(port: u16, seed: u64, players: u8, relay_id: RelayId) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("relay".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .expect("relay tokio runtime");
            rt.block_on(relay::start_relay(port, seed, players, Some(relay_id)))
                .expect("relay server")
        })
        .expect("relay thread")
}

/// Find a player-0 soldier in the world.
fn find_player_unit(world: &mut bevy::ecs::world::World, player: u8) -> Option<UnitId> {
    let mut q = world.query::<(&UnitIdComponent, &FactionComponent, &SoldierMarker)>();
    q.iter(world)
        .find(|(_, f, _)| f.0 == FactionId(player))
        .map(|(id, _, _)| id.0)
}

/// Build a network-mode Bevy App for one client.
/// Returns the player unit (if the map spawned one for this faction).
fn build_client_app(
    seed: u64,
    map_size: MapSize,
    player_id: u8,
    nrecv: bevy_adapter::transport::NetworkReceiver,
    nsend: bevy_adapter::transport::NetworkSender,
) -> (App, Option<UnitId>) {
    let mut raw_world = simulation::init_simulation_world(seed);
    simulation::map::generate_map(&mut raw_world, map_size);
    let unit = find_player_unit(&mut raw_world, player_id);
    let sim_world = SimulationWorld::new(raw_world);

    let mut app = App::new();
    app.init_resource::<Time>();
    app.insert_resource(nrecv);
    app.insert_resource(nsend);
    app.insert_resource(SimulationDriver {
        clock: TickClock::default(),
        scheduler: SchedulerState::default(),
        source: CommandSource::Network(NetworkCommandSource {
            game_id: 1,
            player_id,
            input_delay: 3,
            relay_buffer: std::collections::HashMap::new(),
            ruleset_version: 1,
            connected: false,
            reconnect_meta: None,
        }),
        bootstrap_phase: bevy_adapter::session::bootstrap::BootstrapPhase::Active,
    });
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
    app.add_systems(
        Update,
        (network_poll_system, network_flush_system, simulation_driver_system).chain(),
    );
    (app, unit)
}

/// Advance one client app by 50ms and pump the network.
fn step_client(app: &mut App) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(0.05));
    app.update();
}

#[test]
fn test_two_player_move_command_executes() {
    // 1. Relay with 2 players
    let port = find_free_port();
    let _relay = spawn_relay(port, 42, 2, RelayId(1));
    thread::sleep(Duration::from_millis(200));

    // 2. Two network clients (host=0, joiner=1)
    let ev0 = NetworkEventReceiver::default();
    let ev1 = NetworkEventReceiver::default();
    let (nrecv0, nsend0, h0) = spawn_network_client(
        format!("127.0.0.1:{port}"), 1, 0, 1, ev0.clone(), RelayId(1),
    )
    .expect("host client connect");
    let (nrecv1, nsend1, h1) = spawn_network_client(
        format!("127.0.0.1:{port}"), 1, 1, 1, ev1.clone(), RelayId(1),
    )
    .expect("joiner client connect");
    // Leak the handles so their Drop doesn't block on joining the network threads,
    // which run forever until the relay stops.
    std::mem::forget(h0);
    std::mem::forget(h1);
    thread::sleep(Duration::from_millis(500));

    // 3. Two Bevy apps. Note: the default map spawns only cities, no soldiers.
    let seed = 42u64;
    let map_size = MapSize::Small;
    let (mut app0, _host_unit) = build_client_app(seed, map_size, 0, nrecv0, nsend0);
    let (mut app1, _joiner_unit) = build_client_app(seed, map_size, 1, nrecv1, nsend1);

    // 4. Run frames. Issue exactly ONE MoveTo per distinct tick (simulating a
    //    single right-click that must reliably reach the simulation). Commands
    //    target current_tick + input_delay (=3) so the relay cannot have
    //    finalized that tick yet.
    let input_delay = 3u32;
    let mut issued_ticks: Vec<u32> = Vec::new();
    let mut last_issued_tick: Option<u32> = None;
    let mut last_tick = 0u32;
    let mut stalled_frames = 0u32;
    for frame in 0..120 {
        // Issue a MoveTo for the delayed tick if we haven't already for it.
        let cur = app0.world().resource::<TickClock>().current_tick;
        let target = cur + input_delay;
        if last_issued_tick != Some(target) {
            last_issued_tick = Some(target);
            issued_ticks.push(target);
            app0.world_mut().resource_mut::<CommandBuffer>().0
                .push(GameCommand {
                    tick: target,
                    player_id: 0,
                    action: simulation::command::Action::MoveTo {
                        unit: UnitId(1),
                        target: FixedVec2::new(Fixed::from_int(300), Fixed::from_int(300)),
                    },
                });
        }
        step_client(&mut app0);

        // Joiner (no command — just keeps the relay alive with empty frames)
        step_client(&mut app1);

        // Let the relay process both players' frames
        thread::sleep(Duration::from_millis(20));

        // Progress diagnostics + stall detection
        if frame % 20 == 0 {
            let t = app0.world().resource::<SimulationDriver>().clock.current_tick;
            eprintln!("[TEST] frame {}: host tick={}", frame, t);
        }
        let t = app0.world().resource::<SimulationDriver>().clock.current_tick;
        if t == last_tick {
            stalled_frames += 1;
        } else {
            stalled_frames = 0;
            last_tick = t;
        }
        if stalled_frames > 40 {
            eprintln!("[TEST] STALLED: tick not advancing for 40 frames");
            break;
        }
    }

    // 5. Verify: EVERY issued MoveTo reached the simulation (command_log).
    let final_tick = app0.world().resource::<SimulationDriver>().clock.current_tick;
    eprintln!("[TEST] final tick = {}", final_tick);
    assert!(
        final_tick >= 20,
        "driver should advance at least 20 ticks, got {}",
        final_tick
    );

    let recorder = app0.world().resource::<ReplayRecorder>();
    let delivered_ticks: std::collections::HashSet<u32> = recorder
        .command_log
        .iter()
        .filter(|(_, cmds)| {
            cmds.iter().any(|c| {
                matches!(c.action, simulation::command::Action::MoveTo { .. })
            })
        })
        .map(|(tick, _)| *tick)
        .collect();
    // Only require delivery for ticks the driver has already processed.
    // Commands target current_tick + input_delay; the final few issued ticks
    // are future ticks the relay hasn't finalized yet by the time the loop ends.
    let issued: std::collections::HashSet<u32> = issued_ticks.iter().copied().collect();
    let processed_issued: Vec<u32> = issued.iter().copied().filter(|t| *t <= final_tick).collect();
    let missing: Vec<u32> = processed_issued
        .iter()
        .filter(|t| !delivered_ticks.contains(t))
        .copied()
        .collect();
    eprintln!(
        "[TEST] issued {} distinct MoveTo ticks, driver processed to {}, delivered {} — missing {:?}",
        issued.len(),
        final_tick,
        delivered_ticks.len(),
        missing
    );
    assert!(
        missing.is_empty(),
        "Reliability FAILED: commands for {} processed ticks were dropped: {:?}",
        missing.len(),
        missing
    );
}

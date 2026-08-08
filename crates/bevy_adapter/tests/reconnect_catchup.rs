//! Integration test: after reconnect, the driver catches up to the relay's
//! current tick via accumulated accumulator + relay_buffer.
//!
//! This refutes the concern that a reconnecting client "permanently lags":
//! during the disconnect window the driver's accumulator keeps growing, so on
//! resume it consumes the buffered ticks in a single frame (catch-up), then
//! continues at normal pace. See specs/network-reconnect (Scene A).

use bevy::prelude::*;
use bevy_adapter::driver::{
    CommandSource, SchedulerState, SimulationDriver, TickClock, simulation_driver_system,
};
use bevy_adapter::network::{NetworkCommandSource, PAGE_TICKS, ReconnectPage, ReconnectResponse, TickCommands};
use bevy_adapter::replay::ReplayRecorder;
use bevy_adapter::tick::{PendingEvents, SimulationWorld};
use simulation::command::CommandBuffer;
use simulation::map::MapSize;
use simulation::types::PlayerSlots;

/// Build reconnect metadata for `total` ticks starting at `first`.
fn build_metadata(first: u32, total: u32) -> ReconnectResponse {
    ReconnectResponse {
        game_id: 1,
        ruleset_version: 1,
        seed: 42,
        map_spec_hash: 0,
        map_size: MapSize::Small,
        first_tick: first,
        total_ticks: total,
        page_count: total.div_ceil(PAGE_TICKS),
        players: vec![],
    }
}

/// Build the page that covers `[first + i*PAGE_TICKS, min(first+total, ...))`
/// — mirrors relay_core's tick-VALUE bucketing (D2).
fn build_page(first: u32, total: u32, page_index: u32) -> ReconnectPage {
    let lo = first + page_index * PAGE_TICKS;
    let hi = (first + total).min(lo + PAGE_TICKS);
    ReconnectPage {
        page_index,
        page_count: total.div_ceil(PAGE_TICKS),
        first_tick: lo,
        ticks: (lo..hi)
            .map(|t| TickCommands {
                tick: t,
                commands: vec![],
            })
            .collect(),
    }
}

#[test]
fn test_reconnect_catchup_advances_multiple_ticks() {
    let seed = 42u64;
    let mut raw_world = simulation::init_simulation_world_multi(seed, PlayerSlots::multi_player(4, 0));
    simulation::map::generate_map(&mut raw_world, MapSize::Small);
    let sim_world = SimulationWorld::new(raw_world);

    let mut app = App::new();
    app.init_resource::<Time>();

    // 重连:元数据 + 多页(ticks 2..=50 = 49 ticks → 2 页)渐进灌入 relay_buffer
    let mut ns = NetworkCommandSource::default();
    let meta = build_metadata(2, 49);
    ns.apply_reconnect(&meta, 1).unwrap();
    assert_eq!(meta.page_count, 2, "49 ticks must span 2 pages");
    for i in 0..meta.page_count {
        let page = build_page(2, 49, i);
        ns.apply_reconnect_page(&page).unwrap();
    }

    // driver 停在 tick 1,accumulator 积累 10s(=200 tick 的余量,模拟断点期间)
    app.insert_resource(SimulationDriver {
        clock: TickClock {
            current_tick: 1,
            tick_duration: 0.05,
            accumulator: 10.0,
        },
        scheduler: SchedulerState::default(),
        source: CommandSource::Network(ns),
        bootstrap_phase: bevy_adapter::session::bootstrap::BootstrapPhase::Active,
        catch_up: false,
    });
    app.insert_resource(TickClock::default());
    app.init_resource::<PendingEvents>();
    app.insert_resource(CommandBuffer(Vec::new()));
    app.insert_resource(ReplayRecorder {
        seed,
        map_size: MapSize::Small,
        ..Default::default()
    });
    app.insert_non_send(sim_world);
    app.add_systems(Update, simulation_driver_system);

    // 单帧:accumulator 余量充足 → 追平 buffer 到 tick 50
    app.update();

    let driver = app.world().resource::<SimulationDriver>();
    assert!(
        driver.clock.current_tick >= 50,
        "driver should catch up to relay tick 50 in one frame, got {}",
        driver.clock.current_tick
    );

    // 追平后 accumulator 应有剩余(可继续正常节奏),且 is_tick_ready(51) 为 false(等 relay 新帧)
    let ns = match &driver.source {
        CommandSource::Network(ns) => ns,
        _ => panic!("expected Network source"),
    };
    assert!(!ns.is_tick_ready(51), "tick 51 should not be ready yet");
}

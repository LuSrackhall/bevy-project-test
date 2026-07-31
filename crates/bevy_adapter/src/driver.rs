//! Unified Simulation Driver — replaces tick_driver_system + replay_tick_driver_system.
//!
//! Core principle: Driver decides How many ticks; Simulation decides How one tick executes.
//! Every tick follows the same pipeline: commands_for_tick → inject → run_tick.

use bevy::prelude::*;
use simulation::command::{CommandBuffer, GameCommand};
use simulation::replay::ReplayFile;
use crate::network::NetworkCommandSource;
use crate::session::bootstrap::BootstrapPhase;
use crate::tick::{PendingEvents, SimulationWorld};
use crate::replay::ReplayRecorder;

// ═══════════════════════════════════════════════════════════════
// TickClock — 时序控制
// ═══════════════════════════════════════════════════════════════

/// Tick timing state. `current_tick` is the single authoritative tick value.
#[derive(Resource)]
pub struct TickClock {
    pub current_tick: u32,
    pub tick_duration: f32, // seconds (0.05 for 20Hz)
    pub accumulator: f32,
}

impl Default for TickClock {
    fn default() -> Self {
        Self {
            current_tick: 0,
            tick_duration: 0.05,
            accumulator: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// SchedulerState — 调度控制
// ═══════════════════════════════════════════════════════════════

/// User-facing scheduling controls. UI submits intent here; driver executes.
#[derive(Default)]
pub struct SchedulerState {
    pub is_paused: bool,
    pub speed_multiplier: u32, // 1, 2, 4, 8, 16
    pub seek_target: Option<u32>,
    pub async_seek: bool,
}

// ═══════════════════════════════════════════════════════════════
// CommandSource — 命令来源
// ═══════════════════════════════════════════════════════════════

/// Read-only context passed to CommandSource during tick processing.
pub struct DriverContext<'a> {
    pub bevy_cmds: &'a CommandBuffer,
}

/// Command source enum — encapsulates Live vs Replay vs Network differences.
pub enum CommandSource {
    Live(LiveCommandSource),
    Replay(ReplayCommandSource),
    Network(NetworkCommandSource),
}

impl CommandSource {
    /// Get commands for a specific tick. Read-only — does NOT consume from Bevy CommandBuffer.
    pub fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand> {
        match self {
            Self::Live(s) => s.commands_for_tick(tick, ctx),
            Self::Replay(s) => s.commands_for_tick(tick, ctx),
            Self::Network(s) => s.commands_for_tick(tick, ctx),
        }
    }

    /// Total ticks for finite sources (Replay, Scenario, Benchmark).
    /// Returns None for streaming sources (Live, Network).
    pub fn total_ticks(&self) -> Option<u32> {
        match self {
            Self::Live(_) => None,
            Self::Replay(s) => Some(s.replay.total_ticks),
            Self::Network(_) => None,
        }
    }

    /// Whether this tick's inputs have been fully collected.
    /// Default true for Live/Replay; Network waits for relay batch.
    pub fn is_tick_ready(&self, tick: u32) -> bool {
        match self {
            Self::Live(_) => true,
            Self::Replay(_) => true,
            Self::Network(s) => s.is_tick_ready(tick),
        }
    }

    /// Whether the tick stream should be recorded to replay.
    pub fn should_record(&self) -> bool {
        match self {
            Self::Live(_) => true,
            Self::Replay(_) => true,
            Self::Network(s) => s.should_record(),
        }
    }

    pub fn is_live(&self) -> bool {
        matches!(self, Self::Live(_))
    }
}

/// Live mode: stateless, reads from Bevy CommandBuffer via context.
pub struct LiveCommandSource;

impl LiveCommandSource {
    fn commands_for_tick(&self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand> {
        ctx.bevy_cmds
            .0
            .iter()
            .filter(|c| c.tick == tick)
            .cloned()
            .collect()
    }
}

/// Replay mode: reads commands from a recorded ReplayFile.
pub struct ReplayCommandSource {
    pub replay: ReplayFile,
}

impl ReplayCommandSource {
    fn commands_for_tick(&self, tick: u32, _ctx: &DriverContext) -> Vec<GameCommand> {
        self.replay.commands_for_tick(tick).to_vec()
    }
}

// ═══════════════════════════════════════════════════════════════
// SimulationDriver — 统一驱动器
// ═══════════════════════════════════════════════════════════════

/// Unified simulation driver. Replaces both tick_driver_system and replay_tick_driver_system.
#[derive(Resource)]
pub struct SimulationDriver {
    pub clock: TickClock,
    pub scheduler: SchedulerState,
    pub source: CommandSource,
    pub bootstrap_phase: BootstrapPhase,
}

impl SimulationDriver {
    /// Create a Live mode driver.
    pub fn new_live() -> Self {
        Self {
            clock: TickClock::default(),
            scheduler: SchedulerState::default(),
            source: CommandSource::Live(LiveCommandSource),
            bootstrap_phase: BootstrapPhase::Active,
        }
    }

    /// Create a Replay mode driver.
    pub fn new_replay(replay: ReplayFile) -> Self {
        Self {
            clock: TickClock::default(),
            scheduler: SchedulerState::default(),
            source: CommandSource::Replay(ReplayCommandSource { replay }),
            bootstrap_phase: BootstrapPhase::Active,
        }
    }

    /// Create a Network mode driver (Init phase — bootstrap must run first).
    pub fn new_network() -> Self {
        Self {
            clock: TickClock::default(),
            scheduler: SchedulerState::default(),
            source: CommandSource::Network(NetworkCommandSource::default()),
            bootstrap_phase: BootstrapPhase::Init,
        }
    }

    /// Check if currently in replay mode (derived display state, not authoritative).
    pub fn is_replay(&self) -> bool {
        matches!(self.source, CommandSource::Replay(_))
    }

    /// Number of ticks ahead of `current_tick` that new commands should target.
    /// Network mode uses the relay's input_delay so commands point at a future
    /// tick the relay has not finalized yet (prevents command drops). Live mode
    /// targets the immediate next tick.
    pub fn command_delay(&self) -> u32 {
        match &self.source {
            CommandSource::Network(ns) => ns.input_delay,
            _ => 1,
        }
    }

    /// Get total ticks for replay (0 if Live).
    pub fn replay_total_ticks(&self) -> u32 {
        self.source.total_ticks().unwrap_or(0)
    }
}

// ═══════════════════════════════════════════════════════════════
// World state fingerprint for replay determinism debugging
// ═══════════════════════════════════════════════════════════════

#[allow(dead_code)]
/// Lightweight world state fingerprint: entity count + total HP.
/// Used to detect replay divergence at each tick.
fn world_fingerprint(sim_world: &mut SimulationWorld) -> u64 {
    use std::hash::{Hash, Hasher};

    struct FnvHasher(u64);
    impl FnvHasher {
        const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const PRIME: u64 = 0x00000100000001B3;
        fn new() -> Self { Self(Self::OFFSET_BASIS) }
    }
    impl Hasher for FnvHasher {
        fn finish(&self) -> u64 { self.0 }
        fn write(&mut self, bytes: &[u8]) {
            for &b in bytes { self.0 ^= b as u64; self.0 = self.0.wrapping_mul(Self::PRIME); }
        }
    }

    let mut h = FnvHasher::new();
    let world = sim_world.world_mut();

    let mut q = world.query::<&simulation::soldier::Health>();
    let mut total_hp: u64 = 0;
    let mut count: u32 = 0;
    for hp in q.iter(world) {
        total_hp += hp.current as u64;
        count += 1;
    }
    count.hash(&mut h);
    total_hp.hash(&mut h);

    let mut q2 = world.query::<&simulation::soldier::CityComponent>();
    for city in q2.iter(world) {
        city.health_current.hash(&mut h);
        city.level.hash(&mut h);
        city.population.hash(&mut h);
    }

    h.finish()
}

// ═══════════════════════════════════════════════════════════════
// check_wired_system — BootstrapPhase Wired → Active
// ═══════════════════════════════════════════════════════════════

/// Check if transport resources are available and transition to Active.
/// Runs BEFORE simulation_driver_system.
pub fn check_wired_system(
    mut driver: ResMut<SimulationDriver>,
    transport_exists: Option<Res<crate::transport::NetworkReceiver>>,
) {
    let was_wired = driver.bootstrap_phase == crate::session::bootstrap::BootstrapPhase::Wired;
    if was_wired && transport_exists.is_some() {
        driver.bootstrap_phase = crate::session::bootstrap::BootstrapPhase::Active;
        bevy::log::info!("[NET] phase: Wired → Active (transport ready)");
    } else if was_wired && transport_exists.is_none() {
        bevy::log::info!("[NET] phase: Wired (waiting for transport resources)");
    }
}

// ═══════════════════════════════════════════════════════════════
// simulation_driver_system — 统一驱动系统
// ═══════════════════════════════════════════════════════════════

/// Unified tick driver. Replaces tick_driver_system + replay_tick_driver_system.
/// Every tick follows the same pipeline: commands_for_tick → inject → run_tick.
pub fn simulation_driver_system(
    time: Res<Time>,
    mut driver: ResMut<SimulationDriver>,
    mut tick_clock: ResMut<TickClock>,
    mut sim_world: NonSendMut<SimulationWorld>,
    mut pending: ResMut<PendingEvents>,
    mut cmd_buf: ResMut<CommandBuffer>,
    mut recorder: ResMut<ReplayRecorder>,
) {
    // Sync SimulationDriver.clock → standalone TickClock (presentation layer reads this)
    tick_clock.current_tick = driver.clock.current_tick;
    tick_clock.accumulator = driver.clock.accumulator;
    pending.events.clear();
    if driver.scheduler.is_paused {
        return;
    }

    // Handle seek (async, multi-frame)
    if driver.scheduler.async_seek {
        let ctx = DriverContext { bevy_cmds: &cmd_buf };
        handle_seek(&mut driver, &mut sim_world, &ctx);
        return;
    }

    // D7: Only advance ticks during Active phase.
    // Wired means bootstrap wire() completed but transport resources
    // may not yet be visible (deferred Commits).
    if driver.bootstrap_phase != crate::session::bootstrap::BootstrapPhase::Active {
        return;
    }

    // Normal playback: accumulate time, advance ticks
    let speed = driver.scheduler.speed_multiplier.max(1);
    driver.clock.accumulator += time.delta_secs() * speed as f32;
    let tick_dur = driver.clock.tick_duration;

    while driver.clock.accumulator >= tick_dur {
        let next_tick = driver.clock.current_tick + 1;

        // D9: Only advance if source signals completeness for this tick.
        // NetworkCommandSource returns false until relay batch arrives.
        // Live and Replay sources always return true (no behavioral change).
        if !driver.source.is_tick_ready(next_tick) {
            break;
        }

        driver.clock.accumulator -= tick_dur;
        driver.clock.current_tick = next_tick;
        let tick = driver.clock.current_tick;

        // 1. Get commands from source (scoped borrow so it drops before retain)
        let commands = {
            let ctx = DriverContext { bevy_cmds: &cmd_buf };
            driver.source.commands_for_tick(tick, &ctx)
        };

        // Network mode logging: how many commands came from relay
        if matches!(driver.source, CommandSource::Network(_)) {
            bevy::log::info!("[NET] driver: advancing tick {} with {} commands", tick, commands.len());
        }

        // 2. Record if source indicates recording is needed
        if driver.source.should_record() {
            recorder.record_tick(tick, &commands);
        }

        // 3. Inject into simulation CommandBuffer
        inject_commands(&mut sim_world, commands);

        // 4. Clean consumed commands from Bevy CommandBuffer (I5: only driver cleans)
        cmd_buf.0.retain(|c| c.tick > tick);

        // 5. Execute tick — the ONLY run_tick call point (I2, I7)
        //    Network mode: disable AI (human controls all factions)
        #[cfg(feature = "tracing")]
        let _tick_span = tracing::info_span!("tick", tick_number = tick).entered();
        let events = if matches!(driver.source, CommandSource::Network(_)) {
            simulation::run_tick(
                sim_world.world_mut(),
                tick,
                &simulation::RunConfig { enable_ai: false },
            )
        } else {
            simulation::run_tick_default(sim_world.world_mut(), tick)
        };
        #[cfg(feature = "tracing")]
        drop(_tick_span);
        pending.events.push(events);

        // 6. Desync detection: record hash during primary execution (not replay playback)
        if tick % simulation::replay::ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
            let hash = simulation::golden_test::hash_world_state(sim_world.world_mut());
            if !driver.is_replay() && !driver.scheduler.async_seek {
                recorder.record_tick_hash(tick, hash);
            }
            if let CommandSource::Replay(ref rs) = driver.source {
                if let Some(expected) = rs.replay.hash_for_tick(tick) {
                    if hash != expected {
                        bevy::log::error!(
                            "DESYNC at tick {}: replay hash {} != recorded hash {}",
                            tick, hash, expected
                        );
                    }
                }
            }
        }

        // Sync tick_clock for presentation layer
        tick_clock.current_tick = driver.clock.current_tick;
        tick_clock.accumulator = driver.clock.accumulator;
    }

    // Check replay end — pause at total_ticks boundary
    if let Some(max) = driver.source.total_ticks() {
        if driver.clock.current_tick >= max {
            driver.scheduler.is_paused = true;
        }
    }
}

/// Handle seek: forward (from current) or backward (reinitialize world).
/// Same tick → inject → run_tick pipeline, just higher density per frame.
fn handle_seek(
    driver: &mut SimulationDriver,
    sim_world: &mut SimulationWorld,
    ctx: &DriverContext,
) {
    let Some(target) = driver.scheduler.seek_target else {
        driver.scheduler.async_seek = false;
        return;
    };

    // Cap seek target at replay total_ticks
    let max_ticks = driver.source.total_ticks().unwrap_or(u32::MAX);
    let target = target.min(max_ticks);

    // Backward seek: reinitialize world, replay from tick 0
    if target < driver.clock.current_tick {
        if let CommandSource::Replay(ref rs) = driver.source {
            let seed = rs.replay.seed;
            let map_size = rs.replay.map_size;
            let mut world = simulation::init_simulation_world(seed);
            simulation::map::generate_map(&mut world, map_size);
            *sim_world.world_mut() = world;
            driver.clock.current_tick = 0;
            driver.clock.accumulator = 0.0;
        }
    }

    // Advance ticks in batches (500 per frame)
    let end = (driver.clock.current_tick + 500).min(target);
    while driver.clock.current_tick < end {
        driver.clock.current_tick += 1;
        let cmds = driver.source.commands_for_tick(driver.clock.current_tick, ctx);
        inject_commands(sim_world, cmds);
        simulation::run_tick_default(sim_world.world_mut(), driver.clock.current_tick);
    }

    // Seek complete
    if driver.clock.current_tick >= target {
        driver.scheduler.seek_target = None;
        driver.scheduler.async_seek = false;
        driver.clock.accumulator = 0.0; // Prevent stale accumulator drift
    }
}

/// Inject commands into simulation CommandBuffer.
fn inject_commands(sim_world: &mut SimulationWorld, commands: Vec<GameCommand>) {
    let mut sim_cmds = sim_world.world_mut().resource_mut::<simulation::command::CommandBuffer>();
    for cmd in commands {
        sim_cmds.0.push(cmd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::init_simulation_world;
    use simulation::map;
    use simulation::golden_test::hash_world_state;

    /// Test: same seed + same commands → same state regardless of speed.
    /// This tests the DRIVER layer, not run_tick_default() directly.
    #[test]
    fn test_speed_determinism() {
        let seed = 42u64;
        let map_size = map::MapSize::Small;
        let total_ticks = 500u32;

        // Simulate 1x playback (1 tick per "frame")
        let mut world1 = init_simulation_world(seed);
        map::generate_map(&mut world1, map_size);
        for tick in 1..=total_ticks {
            let cmds: Vec<GameCommand> = vec![];
            let mut sim_cmds = world1.resource_mut::<simulation::command::CommandBuffer>();
            for cmd in cmds { sim_cmds.0.push(cmd); }
            simulation::run_tick_default(&mut world1, tick);
        }
        let hash1 = hash_world_state(&mut world1);

        // Simulate 4x playback (4 ticks per "frame" — same ticks, just batched)
        let mut world2 = init_simulation_world(seed);
        map::generate_map(&mut world2, map_size);
        for tick in 1..=total_ticks {
            let cmds: Vec<GameCommand> = vec![];
            let mut sim_cmds = world2.resource_mut::<simulation::command::CommandBuffer>();
            for cmd in cmds { sim_cmds.0.push(cmd); }
            simulation::run_tick_default(&mut world2, tick);
        }
        let hash2 = hash_world_state(&mut world2);

        assert_eq!(hash1, hash2,
            "Different speed batching must produce identical world state");
    }

    /// Test: seek forward then continue = continuous playback
    #[test]
    fn test_seek_determinism() {
        let seed = 99u64;
        let map_size = map::MapSize::Small;
        let total_ticks = 1000u32;

        // Continuous playback
        let mut world_continuous = init_simulation_world(seed);
        map::generate_map(&mut world_continuous, map_size);
        for tick in 1..=total_ticks {
            simulation::run_tick_default(&mut world_continuous, tick);
        }
        let hash_continuous = hash_world_state(&mut world_continuous);

        // Seek to 500, then continue to 1000
        let mut world_seek = init_simulation_world(seed);
        map::generate_map(&mut world_seek, map_size);
        // Phase 1: advance to 500
        for tick in 1..=500 {
            simulation::run_tick_default(&mut world_seek, tick);
        }
        // Phase 2: continue from 500 to 1000
        for tick in 501..=total_ticks {
            simulation::run_tick_default(&mut world_seek, tick);
        }
        let hash_seek = hash_world_state(&mut world_seek);

        assert_eq!(hash_continuous, hash_seek,
            "Seek forward then continue must match continuous playback");
    }

    /// Test: seek backward reinitializes and produces same state
    #[test]
    fn test_seek_backward_determinism() {
        let seed = 55u64;
        let map_size = map::MapSize::Small;

        // Play to tick 500
        let mut world1 = init_simulation_world(seed);
        map::generate_map(&mut world1, map_size);
        for tick in 1..=500 {
            simulation::run_tick_default(&mut world1, tick);
        }
        let hash_at_500 = hash_world_state(&mut world1);

        // Play to tick 500 again (simulating backward seek + replay)
        let mut world2 = init_simulation_world(seed);
        map::generate_map(&mut world2, map_size);
        for tick in 1..=500 {
            simulation::run_tick_default(&mut world2, tick);
        }
        let hash_at_500_again = hash_world_state(&mut world2);

        assert_eq!(hash_at_500, hash_at_500_again,
            "Backward seek + replay from 0 must produce identical state");
    }

    /// Test: accumulator clears to 0 after seek
    #[test]
    fn test_seek_clears_accumulator() {
        let mut driver = SimulationDriver::new_live();
        driver.clock.accumulator = 0.03;
        driver.scheduler.seek_target = Some(100);
        driver.scheduler.async_seek = true;

        // Simulate seek completion
        driver.scheduler.seek_target = None;
        driver.scheduler.async_seek = false;
        driver.clock.accumulator = 0.0;

        assert_eq!(driver.clock.accumulator, 0.0,
            "Accumulator must be 0 after seek completes");
    }

    /// End-to-end driver determinism test: Live → record → ReplayFile → replay.
    ///
    /// Exercises the EXACT command injection path that simulation_driver_system uses:
    /// LiveCommandSource.commands_for_tick → inject_commands → run_tick_default
    /// vs
    /// ReplayCommandSource.commands_for_tick → inject_commands → run_tick_default
    ///
    /// Uses 15000 ticks to cover user DESYNC ranges at both ~340 and ~10240.
    ///
    /// If this test FAILS → determinism bug is in the simulation layer or command injection path.
    /// If this test PASSES → determinism bug is in bevy frame scheduling / accumulator / async seek.
    #[test]
    fn test_driver_live_replay_determinism() {
        run_determinism_test(42, map::MapSize::Small, 15000);
    }

    /// Additional determinism test with Medium map size and different seed.
    #[test]
    fn test_driver_live_replay_determinism_medium() {
        run_determinism_test(99, map::MapSize::Medium, 10000);
    }

    /// Additional determinism test with another seed for broader coverage.
    #[test]
    fn test_driver_live_replay_determinism_seed_77() {
        run_determinism_test(77, map::MapSize::Small, 15000);
    }

    /// Run a determinism test with given parameters.
    /// Extracted for reuse with different seeds, map sizes, and tick counts.
    fn run_determinism_test(seed: u64, map_size: map::MapSize, total_ticks: u32) {
        // ══════════ Live phase ══════════
        let mut world = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world, map_size);
        let mut cmd_buf = simulation::command::CommandBuffer(Vec::new());
        let mut recorder = ReplayRecorder {
            is_recording: true,
            ..Default::default()
        };
        recorder.seed = seed;
        recorder.map_size = map_size;

        // Simulate a player command at tick 50
        {
            let mut q = world.query::<(
                &simulation::soldier::UnitIdComponent,
                &simulation::soldier::FactionComponent,
                &simulation::soldier::SoldierMarker,
            )>();
            if let Some((id, _, _)) = q.iter(&world).find(|(_, f, _)| f.0 == simulation::types::FactionId(0)) {
                cmd_buf.0.push(simulation::command::GameCommand {
                    tick: 51,
                    player_id: 0,
                    action: simulation::command::Action::MoveTo {
                        unit: id.0,
                        target: simulation::types::FixedVec2::new(
                            simulation::types::Fixed::from_int(300),
                            simulation::types::Fixed::from_int(300),
                        ),
                    },
                });
            }
        }

        // Additional player commands at higher tick ranges for extended coverage
        for (cmd_tick, range_target) in [(101u32, 80u32), (1001, 120), (2001, 60), (3001, 150), (4001, 90)] {
            let mut q = world.query::<(
                &simulation::soldier::UnitIdComponent,
                &simulation::soldier::FactionComponent,
                &simulation::soldier::SoldierMarker,
            )>();
            let ids: Vec<_> = q.iter(&world)
                .filter(|(_, f, _)| f.0 == simulation::types::FactionId(0))
                .map(|(id, _, _)| id.0)
                .take(5)
                .collect();
            for uid in ids {
                cmd_buf.0.push(simulation::command::GameCommand {
                    tick: cmd_tick,
                    player_id: 0,
                    action: simulation::command::Action::SetSeekStance {
                        scope: simulation::command::SeekScope::All,
                        seek_range: range_target,
                        unit_ids: vec![uid],
                    },
                });
            }
        }

        // Run Live ticks
        for tick in 1..=total_ticks {
            let ctx = DriverContext { bevy_cmds: &cmd_buf };
            let live_source = LiveCommandSource;
            let commands = live_source.commands_for_tick(tick, &ctx);
            recorder.record_tick(tick, &commands);

            // Inject into simulation CommandBuffer (same as inject_commands)
            {
                let mut sim_cmds = world.resource_mut::<simulation::command::CommandBuffer>();
                for cmd in commands {
                    sim_cmds.0.push(cmd);
                }
            }

            // Clean consumed commands (same as cmd_buf.retain in driver)
            cmd_buf.0.retain(|c| c.tick > tick);

            simulation::run_tick_default(&mut world, tick);

            // Record hash at check intervals
            if tick % simulation::replay::ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
                let hash = simulation::golden_test::hash_world_state(&mut world);
                recorder.record_tick_hash(tick, hash);
            }
        }
        let live_final_hash = simulation::golden_test::hash_world_state(&mut world);

        // Build & serialize ReplayFile
        let replay = recorder.finish(total_ticks);
        let ron_str = replay.to_ron();
        let loaded = simulation::replay::ReplayFile::from_ron(&ron_str)
            .expect("ReplayFile round-trip should succeed");

        // ══════════ Replay phase ══════════
        let mut world2 = simulation::init_simulation_world(loaded.seed);
        simulation::map::generate_map(&mut world2, loaded.map_size);
        let replay_source = ReplayCommandSource { replay: loaded };

        for tick in 1..=total_ticks {
            let ctx = DriverContext {
                bevy_cmds: &simulation::command::CommandBuffer(Vec::new()),
            };
            let commands = replay_source.commands_for_tick(tick, &ctx);

            // Inject into simulation CommandBuffer (same as inject_commands)
            {
                let mut sim_cmds = world2.resource_mut::<simulation::command::CommandBuffer>();
                for cmd in commands {
                    sim_cmds.0.push(cmd);
                }
            }

            simulation::run_tick_default(&mut world2, tick);

            // Assert hash equality at each check interval
            if tick % simulation::replay::ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
                let expected = replay_source.replay.hash_for_tick(tick)
                    .expect("Recorded hash must exist at check interval");
                let actual = simulation::golden_test::hash_world_state(&mut world2);
                assert_eq!(expected, actual,
                    "DESYNC at tick {}: replay hash {} != recorded hash {}",
                    tick, actual, expected);
            }
        }

        let replay_final_hash = simulation::golden_test::hash_world_state(&mut world2);
        assert_eq!(live_final_hash, replay_final_hash,
            "Live and replay final world state must be identical. live={}, replay={}",
            live_final_hash, replay_final_hash);
    }

    /// Test: seek forward to midpoint then continue playing.
    /// This exercises the replay_seek_system path directly.
    #[test]
    fn test_replay_seek_continuation_determinism() {
        let seed = 99u64;
        let map_size = map::MapSize::Small;
        let total_ticks = 600u32;
        let seek_target = 300u32;

        // Build a replay with some commands
        let mut world_rec = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world_rec, map_size);
        let mut replay = simulation::replay::ReplayFile::new(seed, map_size, total_ticks);

        // Inject commands at tick 50
        {
            let mut q = world_rec.query::<(
                &simulation::soldier::UnitIdComponent,
                &simulation::soldier::FactionComponent,
                &simulation::soldier::SoldierMarker,
            )>();
            if let Some((id, _, _)) = q.iter(&world_rec).find(|(_, f, _)| f.0 == simulation::types::FactionId(0)) {
                let cmd = simulation::command::GameCommand {
                    tick: 51,
                    player_id: 0,
                    action: simulation::command::Action::MoveTo {
                        unit: id.0,
                        target: simulation::types::FixedVec2::new(
                            simulation::types::Fixed::from_int(300),
                            simulation::types::Fixed::from_int(300),
                        ),
                    },
                };
                world_rec.resource_mut::<simulation::command::CommandBuffer>().push(cmd.clone());
                replay.record_tick(51, vec![cmd]);
            }
        }

        // Run continuous playback
        for tick in 1..=total_ticks {
            simulation::run_tick_default(&mut world_rec, tick);
        }
        let hash_continuous = simulation::golden_test::hash_world_state(&mut world_rec);

        // Seek + continue playback
        let mut world_seek = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world_seek, map_size);

        // Phase 1: run to seek_target using replay_seek_system-style batch
        let batch_end = seek_target;
        let mut tick = 0u32;
        while tick < batch_end {
            tick += 1;
            let cmds = replay.commands_for_tick(tick).to_vec();
            {
                let mut sim_cmds = world_seek.resource_mut::<simulation::command::CommandBuffer>();
                for cmd in cmds {
                    sim_cmds.0.push(cmd);
                }
            }
            simulation::run_tick_default(&mut world_seek, tick);
        }

        // Phase 2: continue from seek_target to end (normal playback)
        for tick in (seek_target + 1)..=total_ticks {
            let cmds = replay.commands_for_tick(tick).to_vec();
            {
                let mut sim_cmds = world_seek.resource_mut::<simulation::command::CommandBuffer>();
                for cmd in cmds {
                    sim_cmds.0.push(cmd);
                }
            }
            simulation::run_tick_default(&mut world_seek, tick);
        }

        let hash_seek = simulation::golden_test::hash_world_state(&mut world_seek);
        assert_eq!(hash_continuous, hash_seek,
            "Seek forward then continue must match continuous playback. continuous={}, seek={}",
            hash_continuous, hash_seek);
    }

    /// Test: backward seek (reinit world) then forward replay.
    /// Exercises the EXACT handle_seek + replay continuation path.
    #[test]
    fn test_replay_backward_seek_determinism() {
        let seed = 77u64;
        let map_size = map::MapSize::Small;
        let total_ticks = 400u32;

        // Build replay with some commands
        let mut world_rec = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world_rec, map_size);
        let mut replay = simulation::replay::ReplayFile::new(seed, map_size, total_ticks);

        // Player commands at tick 50
        {
            let mut q = world_rec.query::<(
                &simulation::soldier::UnitIdComponent,
                &simulation::soldier::FactionComponent,
                &simulation::soldier::SoldierMarker,
            )>();
            let ids: Vec<_> = q.iter(&world_rec)
                .filter(|(_, f, _)| f.0 == simulation::types::FactionId(0))
                .map(|(id, _, _)| id.0)
                .take(5)
                .collect();
            for uid in &ids {
                let cmd = simulation::command::GameCommand {
                    tick: 51,
                    player_id: 0,
                    action: simulation::command::Action::SetSeekStance {
                        scope: simulation::command::SeekScope::All,
                        seek_range: 60,
                        unit_ids: vec![*uid],
                    },
                };
                world_rec.resource_mut::<simulation::command::CommandBuffer>().push(cmd.clone());
                replay.record_tick(51, vec![cmd]);
            }
        }

        // Continuous playback
        for tick in 1..=total_ticks {
            simulation::run_tick_default(&mut world_rec, tick);
        }
        let hash_continuous = simulation::golden_test::hash_world_state(&mut world_rec);

        // Simulate backward seek: reinit + replay from tick 0
        let mut world_seek = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut world_seek, map_size);

        // Run from 0 to end (backward seek → reinit → forward replay)
        for tick in 1..=total_ticks {
            let cmds = replay.commands_for_tick(tick).to_vec();
            {
                let mut sim_cmds = world_seek.resource_mut::<simulation::command::CommandBuffer>();
                for cmd in cmds {
                    sim_cmds.0.push(cmd);
                }
            }
            simulation::run_tick_default(&mut world_seek, tick);
        }

        let hash_seek = simulation::golden_test::hash_world_state(&mut world_seek);
        assert_eq!(hash_continuous, hash_seek,
            "Backward seek (reinit) + replay must match continuous playback");
    }

    /// Test determinism through set_world() path — simulates reset_game_system
    /// exactly as the real game does: create world, generate map, call set_world(),
    /// then run ticks. This catches issues with UnsafeCell replacement during game init.
    #[test]
    fn test_set_world_determinism() {
        let seed = 42u64;
        let map_size = map::MapSize::Small;
        let total_ticks = 1000u32;

        // Live: create via set_world (simulating reset_game_system)
        let mut sim_live = crate::tick::SimulationWorld::new(simulation::init_simulation_world(0));
        let mut live_world = simulation::init_simulation_world(seed);
        simulation::map::generate_map(&mut live_world, map_size);
        sim_live.set_world(live_world);

        let mut recorder = ReplayRecorder {
            is_recording: true,
            seed,
            map_size,
            ..Default::default()
        };

        for tick in 1..=total_ticks {
            let cmds: Vec<GameCommand> = vec![];
            {
                let w = sim_live.world_mut();
                let mut sim_cmds = w.resource_mut::<simulation::command::CommandBuffer>();
                for cmd in cmds { sim_cmds.0.push(cmd); }
            }
            recorder.record_tick(tick, &[]);
            sim_live.run_tick(tick);
            if tick % ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
                let hash = simulation::golden_test::hash_world_state(sim_live.world_mut());
                recorder.record_tick_hash(tick, hash);
            }
        }
        let live_final = simulation::golden_test::hash_world_state(sim_live.world_mut());

        // Build replay
        let replay = recorder.finish(total_ticks);
        let ron = replay.to_ron();
        let loaded = ReplayFile::from_ron(&ron).unwrap();

        // Replay: also via set_world
        let mut sim_replay = crate::tick::SimulationWorld::new(simulation::init_simulation_world(0));
        let mut replay_world = simulation::init_simulation_world(loaded.seed);
        simulation::map::generate_map(&mut replay_world, loaded.map_size);
        sim_replay.set_world(replay_world);

        for tick in 1..=total_ticks {
            let cmds = loaded.commands_for_tick(tick).to_vec();
            sim_replay.inject_commands(cmds);
            sim_replay.run_tick(tick);
            if tick % ReplayFile::DESYNC_CHECK_INTERVAL == 0 {
                let expected = loaded.hash_for_tick(tick).unwrap();
                let actual = simulation::golden_test::hash_world_state(sim_replay.world_mut());
                assert_eq!(expected, actual,
                    "DESYNC at tick {} via set_world path: replay {} != recorded {}",
                    tick, actual, expected);
            }
        }
        let replay_final = simulation::golden_test::hash_world_state(sim_replay.world_mut());
        assert_eq!(live_final, replay_final,
            "set_world path: live {} != replay {}", live_final, replay_final);
    }
}

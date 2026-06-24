//! Unified Simulation Driver — replaces tick_driver_system + replay_tick_driver_system.
//!
//! Core principle: Driver decides How many ticks; Simulation decides How one tick executes.
//! Every tick follows the same pipeline: commands_for_tick → inject → run_tick.

use bevy::prelude::*;
use simulation::command::{CommandBuffer, GameCommand};
use simulation::replay::ReplayFile;
use simulation::SimulationEvents;
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

/// Command source enum — encapsulates Live vs Replay differences.
pub enum CommandSource {
    Live(LiveCommandSource),
    Replay(ReplayCommandSource),
}

impl CommandSource {
    /// Get commands for a specific tick. Read-only — does NOT consume from Bevy CommandBuffer.
    pub fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand> {
        match self {
            Self::Live(s) => s.commands_for_tick(tick, ctx),
            Self::Replay(s) => s.commands_for_tick(tick, ctx),
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
}

impl SimulationDriver {
    /// Create a Live mode driver.
    pub fn new_live() -> Self {
        Self {
            clock: TickClock::default(),
            scheduler: SchedulerState::default(),
            source: CommandSource::Live(LiveCommandSource),
        }
    }

    /// Create a Replay mode driver.
    pub fn new_replay(replay: ReplayFile) -> Self {
        Self {
            clock: TickClock::default(),
            scheduler: SchedulerState::default(),
            source: CommandSource::Replay(ReplayCommandSource { replay }),
        }
    }

    /// Check if currently in replay mode (derived display state, not authoritative).
    pub fn is_replay(&self) -> bool {
        matches!(self.source, CommandSource::Replay(_))
    }

    /// Get total ticks for replay (0 if Live).
    pub fn replay_total_ticks(&self) -> u32 {
        match &self.source {
            CommandSource::Replay(rs) => rs.replay.total_ticks,
            _ => 0,
        }
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
    mut sim_world: NonSendMut<SimulationWorld>,
    mut pending: ResMut<PendingEvents>,
    mut cmd_buf: ResMut<CommandBuffer>,
    mut recorder: ResMut<ReplayRecorder>,
) {
    if driver.scheduler.is_paused {
        return;
    }

    // Handle seek (async, multi-frame)
    if driver.scheduler.async_seek {
        let ctx = DriverContext { bevy_cmds: &cmd_buf };
        handle_seek(&mut driver, &mut sim_world, &ctx);
        return;
    }

    // Normal playback: accumulate time, advance ticks
    let speed = driver.scheduler.speed_multiplier.max(1);
    driver.clock.accumulator += time.delta_secs() * speed as f32;
    let tick_dur = driver.clock.tick_duration;

    while driver.clock.accumulator >= tick_dur {
        driver.clock.accumulator -= tick_dur;
        driver.clock.current_tick += 1;
        let tick = driver.clock.current_tick;
        let is_live = driver.source.is_live();
        let is_seeking = driver.scheduler.async_seek;

        // 1. Get commands from source (scoped borrow so it drops before retain)
        let commands = {
            let ctx = DriverContext { bevy_cmds: &cmd_buf };
            driver.source.commands_for_tick(tick, &ctx)
        };

        // 2. Record if Live + recording enabled + not seeking
        if is_live && !is_seeking {
            recorder.record_tick(tick, &commands);
        }

        // 3. Inject into simulation CommandBuffer
        inject_commands(&mut sim_world, commands);

        // 4. Clean consumed commands from Bevy CommandBuffer (I5: only driver cleans)
        cmd_buf.0.retain(|c| c.tick > tick);

        // 5. Execute tick — the ONLY run_tick call point (I2, I7)
        let events = simulation::run_tick(&mut sim_world.0, tick);
        pending.events.push(events);
    }

    // Check replay end
    if let CommandSource::Replay(ref rs) = driver.source {
        if driver.clock.current_tick >= rs.replay.total_ticks {
            // Replay finished — caller should transition to Live or show end screen
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

    // Backward seek: reinitialize world, replay from tick 0
    if target < driver.clock.current_tick {
        if let CommandSource::Replay(ref rs) = driver.source {
            let seed = rs.replay.seed;
            let map_size = rs.replay.map_size;
            let mut world = simulation::init_simulation_world(seed);
            simulation::map::generate_map(&mut world, map_size);
            sim_world.0 = world;
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
        simulation::run_tick(&mut sim_world.0, driver.clock.current_tick);
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
    let mut sim_cmds = sim_world.0.resource_mut::<simulation::command::CommandBuffer>();
    for cmd in commands {
        sim_cmds.0.push(cmd);
    }
}

//! Replay recording and playback for bevy_adapter.
//!
//! Recording: captures external player commands per tick in the tick_driver_system.
//! Playback: replays commands from a ReplayFile, bypassing real-time tick accumulation.

use bevy::prelude::*;
use simulation::command::{CommandBuffer, GameCommand};
use simulation::replay::ReplayFile;
use simulation::map::MapSize;
use simulation::{SimulationEvents, run_tick, init_simulation_world};
use crate::tick::{TickClock, SimulationWorld, PendingEvents};

/// Current game mode — controls which tick driver runs.
#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameMode {
    /// Normal real-time game.
    #[default]
    Live,
    /// Replaying from a recorded file.
    Replay,
}

/// Recording buffer — collects external player commands per tick.
#[derive(Resource, Default)]
pub struct ReplayRecorder {
    pub seed: u64,
    pub map_size: MapSize,
    pub command_log: Vec<(u32, Vec<GameCommand>)>,
    pub is_recording: bool,
}

impl ReplayRecorder {
    /// Record commands for a tick. Call after extracting commands, before injecting into simulation.
    pub fn record_tick(&mut self, tick: u32, commands: &[GameCommand]) {
        if self.is_recording && !commands.is_empty() {
            self.command_log.push((tick, commands.to_vec()));
        }
    }

    /// Finalize and produce a ReplayFile.
    pub fn finish(&self, total_ticks: u32) -> ReplayFile {
        let mut replay = ReplayFile::new(self.seed, self.map_size, total_ticks);
        for (tick, cmds) in &self.command_log {
            replay.record_tick(*tick, cmds.clone());
        }
        replay
    }
}

/// Replay playback controller.
#[derive(Resource)]
pub struct ReplayController {
    pub replay: ReplayFile,
    pub current_tick: u32,
    pub is_paused: bool,
    pub speed_multiplier: u32,  // 1, 2, 4
    pub seek_target: Option<u32>,  // None = play to end
    pub async_seek: bool,          // true = multi-frame seek in progress
}

/// Status exposed to render_view for progress bar display.
#[derive(Resource, Default)]
pub struct ReplayStatus {
    pub is_replay: bool,
    pub is_seeking: bool,
    pub total_ticks: u32,
}

/// Tick driver for replay mode. Runs instead of tick_driver_system when GameMode::Replay.
pub fn replay_tick_driver_system(
    time: Res<Time>,
    mut game_mode: ResMut<GameMode>,
    mut controller: Option<ResMut<ReplayController>>,
    mut sim_world: NonSendMut<SimulationWorld>,
    mut pending: ResMut<PendingEvents>,
    mut tick_clock: ResMut<TickClock>,
    mut commands: ResMut<CommandBuffer>,
) {
    let Some(ref mut ctrl) = controller else {
        return;
    };
    if ctrl.is_paused || ctrl.async_seek { return; }


    let total = ctrl.replay.total_ticks;
    let speed = ctrl.speed_multiplier.max(1);

    // Normal replay: accumulate real time (scaled by speed) and advance ticks
    // Seek is handled by replay_seek_system in render_view (supports backward seek)
    tick_clock.accumulator += time.delta_secs() * speed as f32;
    let tick_dur = tick_clock.tick_duration;

    while tick_clock.accumulator >= tick_dur && ctrl.current_tick < total {
        tick_clock.accumulator -= tick_dur;
        ctrl.current_tick += 1;
        tick_clock.current_tick = ctrl.current_tick;

        let cmds = ctrl.replay.commands_for_tick(ctrl.current_tick).to_vec();
        {
            let mut sim_cmds = sim_world.0.resource_mut::<simulation::command::CommandBuffer>();
            for cmd in cmds { sim_cmds.0.push(cmd); }
        }

        let events = run_tick(&mut sim_world.0, ctrl.current_tick);
        pending.events.push(events);
    }

    // If replay finished, switch back to Live mode
    if ctrl.current_tick >= total && !ctrl.async_seek {
        *game_mode = GameMode::Live;
    }
}

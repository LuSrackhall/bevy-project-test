//! Replay recording and status for bevy_adapter.
//!
//! Recording: captures external player commands per tick in simulation_driver_system.
//! Playback: handled by CommandSource::Replay in driver.rs.

use simulation::command::GameCommand;
use simulation::map::MapSize;
use simulation::replay::ReplayFile;
use bevy::prelude::*;

/// Recording buffer — collects external player commands per tick.
#[derive(Resource, Default)]
pub struct ReplayRecorder {
    pub seed: u64,
    pub map_size: MapSize,
    pub command_log: Vec<(u32, Vec<GameCommand>)>,
    pub tick_hashes: Vec<(u32, u64)>,
    pub is_recording: bool,
}

impl ReplayRecorder {
    /// Record commands for a tick. Call after extracting commands, before injecting into simulation.
    /// Records every tick unconditionally (including empty Vec) to ensure tick alignment
    /// between live recording and replay playback. The ReplayFile::record_tick in finish()
    /// filters non-empty for file format optimization.
    pub fn record_tick(&mut self, tick: u32, commands: &[GameCommand]) {
        if self.is_recording {
            self.command_log.push((tick, commands.to_vec()));
        }
    }

    /// Record world state hash for desync detection.
    pub fn record_tick_hash(&mut self, tick: u32, hash: u64) {
        if self.is_recording {
            self.tick_hashes.push((tick, hash));
        }
    }

    /// Finalize and produce a ReplayFile.
    pub fn finish(&self, total_ticks: u32) -> ReplayFile {
        let mut replay = ReplayFile::new(self.seed, self.map_size, total_ticks);
        for (tick, cmds) in &self.command_log {
            replay.record_tick(*tick, cmds.clone());
        }
        for (tick, hash) in &self.tick_hashes {
            replay.record_tick_hash(*tick, *hash);
        }
        replay
    }
}

/// Status exposed to render_view for progress bar display.
/// is_replay is a derived display state (from SimulationDriver.source), not authoritative.
#[derive(Resource, Default)]
pub struct ReplayStatus {
    pub is_replay: bool,
    pub is_seeking: bool,
    pub total_ticks: u32,
}

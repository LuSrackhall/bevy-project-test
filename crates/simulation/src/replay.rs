//! Replay data structures — pure data, no I/O or engine concepts.

use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use crate::command::GameCommand;
use crate::map::MapSize;

/// A recorded replay file. Contains everything needed to reconstruct a simulation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayFile {
    /// File format version. Starts at 1.
    pub format_version: u32,
    /// RNG seed used to initialize the simulation.
    pub seed: u64,
    /// Map size preset used for this game.
    pub map_size: MapSize,
    /// Total number of ticks in this replay (for progress bar).
    pub total_ticks: u32,
    /// Commands per tick. Only external player commands are recorded;
    /// AI commands are deterministic and regenerated from seed.
    pub commands_per_tick: BTreeMap<u32, Vec<GameCommand>>,
}

impl ReplayFile {
    /// Current format version.
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new ReplayFile.
    pub fn new(seed: u64, map_size: MapSize, total_ticks: u32) -> Self {
        Self {
            format_version: Self::CURRENT_VERSION,
            seed,
            map_size,
            total_ticks,
            commands_per_tick: BTreeMap::new(),
        }
    }

    /// Record commands for a specific tick.
    pub fn record_tick(&mut self, tick: u32, commands: Vec<GameCommand>) {
        if !commands.is_empty() {
            self.commands_per_tick.insert(tick, commands);
        }
    }

    /// Get commands for a specific tick (empty slice if none).
    pub fn commands_for_tick(&self, tick: u32) -> &[GameCommand] {
        self.commands_per_tick.get(&tick).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Serialize to RON format.
    pub fn to_ron(&self) -> String {
        ron::to_string(self).expect("Failed to serialize ReplayFile to RON")
    }

    /// Deserialize from RON format.
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        let file: ReplayFile = ron::from_str(ron_str)
            .map_err(|e| format!("Failed to parse replay file: {}", e))?;
        if file.format_version != Self::CURRENT_VERSION {
            return Err(format!(
                "Replay format version mismatch: file={}, supported={}",
                file.format_version, Self::CURRENT_VERSION
            ));
        }
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use crate::command::*;

    #[test]
    fn test_replay_file_roundtrip_ron() {
        let mut replay = ReplayFile::new(42, MapSize::Small, 100);
        replay.record_tick(5, vec![
            GameCommand { tick: 5, player_id: 0, action: Action::MoveTo { unit: UnitId(1), target: FixedVec2::new(Fixed::from_int(100), Fixed::from_int(200)) } },
        ]);
        replay.record_tick(10, vec![
            GameCommand { tick: 10, player_id: 0, action: Action::Attack { unit: UnitId(1), target: UnitId(2) } },
            GameCommand { tick: 10, player_id: 0, action: Action::NoOp },
        ]);

        let ron_str = replay.to_ron();
        let loaded = ReplayFile::from_ron(&ron_str).unwrap();

        assert_eq!(loaded.format_version, 1);
        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.total_ticks, 100);
        assert_eq!(loaded.commands_for_tick(5).len(), 1);
        assert_eq!(loaded.commands_for_tick(10).len(), 2);
        assert_eq!(loaded.commands_for_tick(99).len(), 0);
    }

    #[test]
    fn test_replay_file_version_mismatch() {
        let mut replay = ReplayFile::new(42, MapSize::Small, 100);
        replay.format_version = 99;
        let ron_str = replay.to_ron();
        let result = ReplayFile::from_ron(&ron_str);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("version mismatch"));
    }

    #[test]
    fn test_replay_file_empty_commands() {
        let replay = ReplayFile::new(42, MapSize::Medium, 0);
        assert_eq!(replay.commands_for_tick(0).len(), 0);
    }
}

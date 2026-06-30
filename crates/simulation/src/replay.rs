//! Replay data structures — pure data, no I/O or engine concepts.

use crate::command::GameCommand;
use crate::map::MapSize;
use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A recorded replay file. Contains everything needed to reconstruct a simulation.
#[derive(Clone, Debug, Resource, Serialize, Deserialize)]
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
        self.commands_per_tick
            .get(&tick)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Serialize to RON format.
    pub fn to_ron(&self) -> String {
        ron::to_string(self).expect("Failed to serialize ReplayFile to RON")
    }

    /// Deserialize from RON format.
    pub fn from_ron(ron_str: &str) -> Result<Self, String> {
        let file: ReplayFile =
            ron::from_str(ron_str).map_err(|e| format!("Failed to parse replay file: {}", e))?;
        if file.format_version != Self::CURRENT_VERSION {
            return Err(format!(
                "Replay format version mismatch: file={}, supported={}",
                file.format_version,
                Self::CURRENT_VERSION
            ));
        }
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::*;
    use crate::golden_test::hash_world_state;
    use crate::init_simulation_world;
    use crate::map;
    use crate::run_tick_default;
    use crate::soldier::*;
    use crate::types::*;

    #[test]
    fn test_replay_file_roundtrip_ron() {
        let mut replay = ReplayFile::new(42, MapSize::Small, 100);
        replay.record_tick(
            5,
            vec![GameCommand {
                tick: 5,
                player_id: 0,
                action: Action::MoveTo {
                    unit: UnitId(1),
                    target: FixedVec2::new(Fixed::from_int(100), Fixed::from_int(200)),
                },
            }],
        );
        replay.record_tick(
            10,
            vec![
                GameCommand {
                    tick: 10,
                    player_id: 0,
                    action: Action::Attack {
                        unit: UnitId(1),
                        target: UnitId(2),
                    },
                },
                GameCommand {
                    tick: 10,
                    player_id: 0,
                    action: Action::NoOp,
                },
            ],
        );

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

    /// End-to-end replay test: record → serialize → deserialize → replay → verify determinism.
    #[test]
    fn test_e2e_replay_determinism() {
        let seed = 12345u64;
        let map_size = MapSize::Small;
        let total_ticks = 500u32;

        // --- Phase 1: Record ---
        let mut world1 = init_simulation_world(seed);
        map::generate_map(&mut world1, map_size);
        let mut replay = ReplayFile::new(seed, map_size, total_ticks);

        for tick in 1..=total_ticks {
            // Simulate a player command at tick 50: move a player soldier
            if tick == 50 {
                let mut q = world1.query::<(&UnitIdComponent, &FactionComponent, &SoldierMarker)>();
                if let Some((id, _fac, _)) =
                    q.iter(&world1).find(|(_, f, _)| f.0 == Faction::Player)
                {
                    let uid = id.0;
                    let target = FixedVec2::new(Fixed::from_int(300), Fixed::from_int(300));
                    let cmd = GameCommand {
                        tick: 51,
                        player_id: 0,
                        action: Action::MoveTo { unit: uid, target },
                    };
                    world1.resource_mut::<CommandBuffer>().push(cmd.clone());
                    replay.record_tick(51, vec![cmd]);
                }
            }
            run_tick_default(&mut world1, tick);
        }
        let hash1 = hash_world_state(&mut world1);

        // --- Phase 2: Serialize → Deserialize ---
        let ron_str = replay.to_ron();
        let loaded_replay = ReplayFile::from_ron(&ron_str).unwrap();
        assert_eq!(loaded_replay.seed, seed);
        assert_eq!(loaded_replay.total_ticks, total_ticks);
        assert_eq!(loaded_replay.commands_for_tick(51).len(), 1);

        // --- Phase 3: Replay from deserialized file ---
        let mut world2 = init_simulation_world(loaded_replay.seed);
        map::generate_map(&mut world2, loaded_replay.map_size);

        for tick in 1..=total_ticks {
            // Inject recorded commands
            let cmds = loaded_replay.commands_for_tick(tick);
            for cmd in cmds {
                world2.resource_mut::<CommandBuffer>().push(cmd.clone());
            }
            run_tick_default(&mut world2, tick);
        }
        let hash2 = hash_world_state(&mut world2);

        // --- Phase 4: Verify ---
        assert_eq!(hash1, hash2,
            "Replay (record → serialize → deserialize → replay) must produce identical world state. \
             hash1={}, hash2={}", hash1, hash2);
    }

    /// End-to-end test with no external commands (AI-only determinism).
    #[test]
    fn test_e2e_replay_ai_only() {
        let seed = 99999u64;
        let map_size = MapSize::Medium;
        let total_ticks = 1000u32;

        // Record (no external commands)
        let mut world1 = init_simulation_world(seed);
        map::generate_map(&mut world1, map_size);
        let replay = ReplayFile::new(seed, map_size, total_ticks);

        for tick in 1..=total_ticks {
            run_tick_default(&mut world1, tick);
        }
        let hash1 = hash_world_state(&mut world1);

        // Replay (no commands to inject)
        let ron_str = replay.to_ron();
        let loaded = ReplayFile::from_ron(&ron_str).unwrap();
        let mut world2 = init_simulation_world(loaded.seed);
        map::generate_map(&mut world2, loaded.map_size);

        for tick in 1..=total_ticks {
            run_tick_default(&mut world2, tick);
        }
        let hash2 = hash_world_state(&mut world2);

        assert_eq!(
            hash1, hash2,
            "AI-only replay must produce identical world state"
        );
    }

    /// Test: replay with seek produces same final state as continuous replay.
    /// This catches accumulator drift and seek-related determinism bugs.
    #[test]
    fn test_seek_determinism() {
        let seed = 54321u64;
        let map_size = MapSize::Small;
        let total_ticks = 2000u32;

        // Create replay with some commands
        let mut world_rec = init_simulation_world(seed);
        map::generate_map(&mut world_rec, map_size);
        let mut replay = ReplayFile::new(seed, map_size, total_ticks);

        for tick in 1..=total_ticks {
            if tick == 100 {
                let mut q =
                    world_rec.query::<(&UnitIdComponent, &FactionComponent, &SoldierMarker)>();
                if let Some((id, _fac, _)) =
                    q.iter(&world_rec).find(|(_, f, _)| f.0 == Faction::Player)
                {
                    let cmd = GameCommand {
                        tick: 101,
                        player_id: 0,
                        action: Action::MoveTo {
                            unit: id.0,
                            target: FixedVec2::new(Fixed::from_int(300), Fixed::from_int(300)),
                        },
                    };
                    world_rec.resource_mut::<CommandBuffer>().push(cmd.clone());
                    replay.record_tick(101, vec![cmd]);
                }
            }
            run_tick_default(&mut world_rec, tick);
        }
        let hash_continuous = hash_world_state(&mut world_rec);

        // Replay with forward seek at tick 500, then continue to end
        let mut world_seek = init_simulation_world(seed);
        map::generate_map(&mut world_seek, map_size);
        let seek_target = 500u32;

        // Phase 1: seek forward to tick 500
        for tick in 1..=seek_target {
            let cmds = replay.commands_for_tick(tick).to_vec();
            for cmd in cmds {
                world_seek.resource_mut::<CommandBuffer>().push(cmd);
            }
            run_tick_default(&mut world_seek, tick);
        }

        // Phase 2: continue playback from tick 500 to end
        for tick in (seek_target + 1)..=total_ticks {
            let cmds = replay.commands_for_tick(tick).to_vec();
            for cmd in cmds {
                world_seek.resource_mut::<CommandBuffer>().push(cmd);
            }
            run_tick_default(&mut world_seek, tick);
        }

        let hash_seek = hash_world_state(&mut world_seek);
        assert_eq!(
            hash_continuous, hash_seek,
            "Replay with seek must produce identical state as continuous replay"
        );
    }


    /// Large-scale replay determinism: many units, multiple commands, 500 ticks.
    /// Catches HashMap iteration order non-determinism that simple tests miss.
    #[test]
    fn test_large_scale_replay_determinism() {
        let seed = 42u64;
        let map_size = MapSize::Small;
        let total_ticks = 500u32;

        // --- Record phase ---
        let mut world1 = init_simulation_world(seed);
        map::generate_map(&mut world1, map_size);
        let mut replay = ReplayFile::new(seed, map_size, total_ticks);

        for tick in 1..=total_ticks {
            // Issue seek commands at tick 10
            if tick == 10 {
                let mut cmds = Vec::new();
                let mut q = world1.query::<(&UnitIdComponent, &FactionComponent, &SoldierMarker)>();
                let soldier_uids: Vec<UnitId> = q.iter(&world1)
                    .filter(|(_, f, _)| f.0 == Faction::Player)
                    .map(|(id, _, _)| id.0)
                    .collect();
                for uid in soldier_uids {
                    cmds.push(GameCommand {
                        tick: 11,
                        player_id: 0,
                        action: Action::SetSeekStance {
                            scope: crate::command::SeekScope::All,
                            seek_range: 60,
                            unit_ids: vec![uid],
                        },
                    });
                }
                for cmd in &cmds {
                    world1.resource_mut::<CommandBuffer>().push(cmd.clone());
                }
                replay.record_tick(11, cmds);
            }

            // Issue move commands at tick 100
            if tick == 100 {
                let mut cmds = Vec::new();
                let mut q = world1.query::<(&UnitIdComponent, &FactionComponent, &SoldierMarker)>();
                let soldier_uids: Vec<UnitId> = q.iter(&world1)
                    .filter(|(_, f, _)| f.0 == Faction::Player)
                    .take(50)
                    .map(|(id, _, _)| id.0)
                    .collect();
                for uid in soldier_uids {
                    cmds.push(GameCommand {
                        tick: 101,
                        player_id: 0,
                        action: Action::MoveTo {
                            unit: uid,
                            target: FixedVec2::new(Fixed::from_int(200), Fixed::from_int(200)),
                        },
                    });
                }
                for cmd in &cmds {
                    world1.resource_mut::<CommandBuffer>().push(cmd.clone());
                }
                replay.record_tick(101, cmds);
            }

            run_tick_default(&mut world1, tick);
        }
        let hash1 = hash_world_state(&mut world1);

        // --- Replay phase ---
        let ron_str = replay.to_ron();
        let loaded = ReplayFile::from_ron(&ron_str).unwrap();
        let mut world2 = init_simulation_world(loaded.seed);
        map::generate_map(&mut world2, loaded.map_size);

        for tick in 1..=total_ticks {
            let cmds = loaded.commands_for_tick(tick).to_vec();
            for cmd in cmds {
                world2.resource_mut::<CommandBuffer>().push(cmd);
            }
            run_tick_default(&mut world2, tick);
        }
        let hash2 = hash_world_state(&mut world2);

        assert_eq!(
            hash1, hash2,
            "Large-scale replay determinism failed. hash1={}, hash2={}", hash1, hash2
        );
    }
}

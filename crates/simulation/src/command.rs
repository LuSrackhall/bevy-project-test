//! Command system — GameCommand, Action, CommandBuffer.
//!
//! All player (and AI) actions flow through the command pipeline.
//! Simulation systems consume command snapshots, never read input directly.

use crate::types::{Fixed, FixedVec2, ShieldState, SoldierType, UnitId};
use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════
// SeekScope + SeekDirective
// ═══════════════════════════════════════════════════════════════

/// Scope of a seek-stance command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeekScope {
    /// Apply to all friendly units.
    All,
    /// Apply to a specific soldier type.
    ByType(SoldierType),
}

/// A seek directive recorded in the global directive resource.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeekDirective {
    pub scope: SeekScope,
    pub seek_range: u32,
    pub issue_tick: u32,
}

// ═══════════════════════════════════════════════════════════════
// Action: ECS-style fine-grained commands
// ═══════════════════════════════════════════════════════════════

/// A single atomic action issued to one unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Move a unit to a world position.
    MoveTo { unit: UnitId, target: FixedVec2 },

    /// Force-move a unit (skip auto-engagement en route).
    ForceMove { unit: UnitId, target: FixedVec2 },

    /// Attack a target unit.
    Attack { unit: UnitId, target: UnitId },

    /// Return to a friendly city (for healing/upgrading).
    ReturnToCity { unit: UnitId, city: UnitId },

    /// Set shield state for an infantry unit.
    SetShield { unit: UnitId, state: ShieldState },

    /// Change spawn type for a city.
    SetSpawnType {
        city: UnitId,
        soldier_type: SoldierType,
    },

    /// Set seek stance: enable/disable auto-engagement with configurable range.
    /// - scope + seek_range: for All / ByType global directives
    /// - unit_ids: for per-selection commands (overrides per-unit SeekStance)
    SetSeekStance {
        scope: SeekScope,
        seek_range: u32,
        unit_ids: Vec<UnitId>,
    },

    /// No operation — placeholder for missing commands in a tick.
    NoOp,
}

impl Action {
    /// 显式排序标签，用于同一 Tick 内命令的确定性排序。
    /// 禁止依赖 Rust 枚举隐式判别值，防止跨编译环境导致执行顺序分歧。
    /// 新增变体时必须分配新标签，不得复用已有值。
    pub const fn sort_tag(&self) -> u8 {
        match self {
            Action::NoOp => 0,
            Action::MoveTo { .. } => 1,
            Action::ForceMove { .. } => 2,
            Action::Attack { .. } => 3,
            Action::ReturnToCity { .. } => 4,
            Action::SetShield { .. } => 5,
            Action::SetSpawnType { .. } => 6,
            Action::SetSeekStance { .. } => 7,
        }
    }

    /// Number of `Action` variants. MUST be bumped when a variant is added.
    ///
    /// `all_action_variants()` asserts its corpus length against this value, so
    /// a variant that reaches the enum and `sort_tag()` but is forgotten in the
    /// corpus fails a test instead of silently shipping untested.
    pub const VARIANT_COUNT: usize = 8;
}

/// One instance of every `Action` variant, for cross-layer serialization tests.
///
/// Exhaustiveness is enforced two ways:
///   1. `sample()` matches with **no wildcard arm**, so adding a variant breaks
///      the build here and forces the new variant to be handled.
///   2. the length assertion catches a variant that `sample()` handles but the
///      seed list (which drives iteration) omits.
///
/// This matters because the two encodings fail differently: replays use RON
/// (self-describing, forgiving) while the network path uses bincode (positional,
/// unforgiving). A variant can be replay-clean and wire-broken at the same time,
/// and only a corpus that provably covers every variant closes that gap.
pub fn all_action_variants() -> Vec<Action> {
    fn sample(action: Action) -> Action {
        match action {
            Action::MoveTo { .. } => Action::MoveTo {
                unit: UnitId(11),
                target: FixedVec2::new(Fixed::from_int(7), Fixed::from_int(-9)),
            },
            Action::ForceMove { .. } => Action::ForceMove {
                unit: UnitId(12),
                target: FixedVec2::new(Fixed::from_int(-3), Fixed::from_int(4)),
            },
            Action::Attack { .. } => Action::Attack {
                unit: UnitId(13),
                target: UnitId(14),
            },
            Action::ReturnToCity { .. } => Action::ReturnToCity {
                unit: UnitId(15),
                city: UnitId(16),
            },
            Action::SetShield { .. } => Action::SetShield {
                unit: UnitId(17),
                state: ShieldState::Blocking,
            },
            Action::SetSpawnType { .. } => Action::SetSpawnType {
                city: UnitId(18),
                soldier_type: SoldierType::Cavalry,
            },
            Action::SetSeekStance { .. } => Action::SetSeekStance {
                scope: SeekScope::ByType(SoldierType::Archer),
                seek_range: 42,
                unit_ids: vec![UnitId(19), UnitId(20)],
            },
            Action::NoOp => Action::NoOp,
        }
    }

    // Seed list drives iteration; `sample()` replaces every field.
    let seeds = [
        Action::MoveTo {
            unit: UnitId(0),
            target: FixedVec2::ZERO,
        },
        Action::ForceMove {
            unit: UnitId(1),
            target: FixedVec2::ZERO,
        },
        Action::Attack {
            unit: UnitId(2),
            target: UnitId(3),
        },
        Action::ReturnToCity {
            unit: UnitId(4),
            city: UnitId(5),
        },
        Action::SetShield {
            unit: UnitId(6),
            state: ShieldState::Normal,
        },
        Action::SetSpawnType {
            city: UnitId(7),
            soldier_type: SoldierType::Militia,
        },
        Action::SetSeekStance {
            scope: SeekScope::All,
            seek_range: 0,
            unit_ids: Vec::new(),
        },
        Action::NoOp,
    ];
    let variants: Vec<Action> = seeds.into_iter().map(sample).collect();

    assert_eq!(
        variants.len(),
        Action::VARIANT_COUNT,
        "Action corpus is stale — bump Action::VARIANT_COUNT and add the new variant to `seeds`"
    );

    // sort_tag() is unique per variant, so a duplicate here means the seed list
    // repeats one variant instead of covering a distinct one.
    let mut tags: Vec<u8> = variants.iter().map(Action::sort_tag).collect();
    tags.sort_unstable();
    tags.dedup();
    assert_eq!(
        tags.len(),
        variants.len(),
        "Action corpus repeats a variant instead of covering every one"
    );

    variants
}

// ═══════════════════════════════════════════════════════════════
// GlobalSeekDirective
// ═══════════════════════════════════════════════════════════════

/// Records the most recent global seek directives issued by the player.
/// Newly spawned units consult this resource to inherit seek stance.
#[derive(Clone, Debug, Default, Resource)]
pub struct GlobalSeekDirective(pub Vec<SeekDirective>);

// ═══════════════════════════════════════════════════════════════
// GameCommand + CommandBuffer
// ═══════════════════════════════════════════════════════════════

/// A command issued by a player for a specific tick.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameCommand {
    /// The tick this command should be consumed.
    pub tick: u32,
    /// Which player issued this command (0 = Player, 1+ = AI/other).
    pub player_id: u8,
    /// The action to execute.
    pub action: Action,
}

/// Buffer of pending commands to be consumed in future ticks.
#[derive(Clone, Debug, Default, Resource)]
pub struct CommandBuffer(pub Vec<GameCommand>);

impl CommandBuffer {
    /// Extract all commands for a specific tick.
    pub fn take_for_tick(&mut self, tick: u32) -> Vec<GameCommand> {
        let mut remaining = Vec::new();
        let mut taken = Vec::new();
        for cmd in self.0.drain(..) {
            if cmd.tick == tick {
                taken.push(cmd);
            } else {
                remaining.push(cmd);
            }
        }
        self.0 = remaining;
        taken
    }

    /// Push a single command.
    pub fn push(&mut self, cmd: GameCommand) {
        self.0.push(cmd);
    }

    /// Check if any command exists for a given tick.
    pub fn has_commands_for(&self, tick: u32) -> bool {
        self.0.iter().any(|c| c.tick == tick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_buffer_take_for_tick() {
        let mut buf = CommandBuffer(vec![
            GameCommand {
                tick: 1,
                player_id: 0,
                action: Action::NoOp,
            },
            GameCommand {
                tick: 1,
                player_id: 0,
                action: Action::NoOp,
            },
            GameCommand {
                tick: 2,
                player_id: 0,
                action: Action::NoOp,
            },
        ]);
        let tick1 = buf.take_for_tick(1);
        assert_eq!(tick1.len(), 2);
        assert_eq!(buf.0.len(), 1); // tick 2 remains
        assert_eq!(buf.0[0].tick, 2);
    }

    #[test]
    fn test_command_buffer_has_commands() {
        let mut buf = CommandBuffer(vec![GameCommand {
            tick: 5,
            player_id: 0,
            action: Action::NoOp,
        }]);
        assert!(buf.has_commands_for(5));
        assert!(!buf.has_commands_for(3));
        buf.take_for_tick(5);
        assert!(!buf.has_commands_for(5));
    }

    #[test]
    fn test_sort_tag_returns_fixed_values() {
        assert_eq!(Action::NoOp.sort_tag(), 0);
        assert_eq!(
            Action::MoveTo {
                unit: UnitId(1),
                target: FixedVec2::ZERO
            }
            .sort_tag(),
            1
        );
        assert_eq!(
            Action::ForceMove {
                unit: UnitId(1),
                target: FixedVec2::ZERO
            }
            .sort_tag(),
            2
        );
        assert_eq!(
            Action::Attack {
                unit: UnitId(1),
                target: UnitId(2)
            }
            .sort_tag(),
            3
        );
        assert_eq!(
            Action::ReturnToCity {
                unit: UnitId(1),
                city: UnitId(2)
            }
            .sort_tag(),
            4
        );
        assert_eq!(
            Action::SetShield {
                unit: UnitId(1),
                state: crate::types::ShieldState::Normal
            }
            .sort_tag(),
            5
        );
        assert_eq!(
            Action::SetSpawnType {
                city: UnitId(1),
                soldier_type: crate::types::SoldierType::Militia
            }
            .sort_tag(),
            6
        );
        assert_eq!(
            Action::SetSeekStance {
                scope: SeekScope::All,
                seek_range: 100,
                unit_ids: vec![]
            }
            .sort_tag(),
            7
        );
    }

    #[test]
    fn test_sort_tag_deterministic_ordering() {
        let mut commands = [
            GameCommand {
                tick: 1,
                player_id: 1,
                action: Action::Attack {
                    unit: UnitId(1),
                    target: UnitId(2),
                },
            },
            GameCommand {
                tick: 1,
                player_id: 0,
                action: Action::MoveTo {
                    unit: UnitId(3),
                    target: FixedVec2::ZERO,
                },
            },
            GameCommand {
                tick: 1,
                player_id: 0,
                action: Action::Attack {
                    unit: UnitId(3),
                    target: UnitId(4),
                },
            },
        ];
        commands.sort_by_key(|c| (c.player_id, c.action.sort_tag()));
        assert_eq!(commands[0].player_id, 0);
        assert_eq!(commands[0].action.sort_tag(), 1); // MoveTo
        assert_eq!(commands[1].player_id, 0);
        assert_eq!(commands[1].action.sort_tag(), 3); // Attack
        assert_eq!(commands[2].player_id, 1);
        assert_eq!(commands[2].action.sort_tag(), 3); // Attack
    }

    #[test]
    fn test_local_player_id_fallback() {
        // LocalPlayerId default is 0 — matches single-player expectation
        assert_eq!(crate::types::LocalPlayerId::default().0, 0);
    }

    #[test]
    fn every_action_variant_survives_ron_round_trip() {
        // Replay encoding. RON is self-describing, so this is the *forgiving*
        // half of the pair — the bincode half lives in
        // `bevy_adapter/tests/action_wire_format.rs`. Both walk the same corpus,
        // so a new variant cannot ship having passed one encoding and never
        // having been tried on the other.
        for action in all_action_variants() {
            let cmd = GameCommand {
                tick: 7,
                player_id: 3,
                action: action.clone(),
            };
            let encoded = ron::to_string(&cmd).expect("RON encode");
            let decoded: GameCommand = ron::from_str(&encoded).expect("RON decode");
            assert_eq!(cmd, decoded, "RON round-trip changed {action:?}");
        }
    }
}

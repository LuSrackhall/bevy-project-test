//! Command system — GameCommand, Action, CommandBuffer.
//!
//! All player (and AI) actions flow through the command pipeline.
//! Simulation systems consume command snapshots, never read input directly.

use bevy_ecs::prelude::Resource;
use crate::types::{FixedVec2, ShieldState, SoldierType, UnitId};

// ═══════════════════════════════════════════════════════════════
// Action: ECS-style fine-grained commands
// ═══════════════════════════════════════════════════════════════

/// A single atomic action issued to one unit.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    SetSpawnType { city: UnitId, soldier_type: SoldierType },

    /// No operation — placeholder for missing commands in a tick.
    NoOp,
}

// ═══════════════════════════════════════════════════════════════
// GameCommand + CommandBuffer
// ═══════════════════════════════════════════════════════════════

/// A command issued by a player for a specific tick.
#[derive(Clone, Debug, PartialEq, Eq)]
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
            GameCommand { tick: 1, player_id: 0, action: Action::NoOp },
            GameCommand { tick: 1, player_id: 0, action: Action::NoOp },
            GameCommand { tick: 2, player_id: 0, action: Action::NoOp },
        ]);
        let tick1 = buf.take_for_tick(1);
        assert_eq!(tick1.len(), 2);
        assert_eq!(buf.0.len(), 1); // tick 2 remains
        assert_eq!(buf.0[0].tick, 2);
    }

    #[test]
    fn test_command_buffer_has_commands() {
        let mut buf = CommandBuffer(vec![
            GameCommand { tick: 5, player_id: 0, action: Action::NoOp },
        ]);
        assert!(buf.has_commands_for(5));
        assert!(!buf.has_commands_for(3));
        buf.take_for_tick(5);
        assert!(!buf.has_commands_for(5));
    }
}

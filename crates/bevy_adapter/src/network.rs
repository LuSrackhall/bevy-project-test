//! Network protocol types for RTS CommandStream Protocol v1.0.
//!
//! Data structures for relay-backed deterministic lockstep multiplayer.
//! See: openspec/changes/network-command-stream/brainstorm-spec.md §3.1

use serde::{Deserialize, Serialize};
use simulation::command::GameCommand;

/// Player state in an active game session (relay view).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerState {
    /// Player is actively connected and submitting frames.
    Active { player_id: u8 },
    /// Player has disconnected; NoOp will be injected.
    Disconnected { player_id: u8 },
}

/// Game initialization parameters, seeded and versioned for determinism.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameInitParams {
    pub game_id: u64,
    pub seed: u64,
    pub ruleset_version: u32,
    pub map_spec_hash: u64,
    pub players: Vec<u8>, // player_ids
}

// ═══════════════════════════════════════════════════════════════
// TickCommands — simulation artifact (pure, no network metadata)
// ═══════════════════════════════════════════════════════════════

/// Canonical command batch for one tick.
///
/// TickCommands is a **simulation artifact** — it contains only tick number
/// and the sorted, finalized commands. It does NOT carry transport metadata
/// (game_id, relay_ts, etc.). It is used for:
///   - Replay recording and playback
///   - Reconnect log transfer
///   - Seek recovery
///
/// Properties:
///   - Commands are pre-sorted by (player_id, sort_tag) at relay finalization
///   - TickCommands is a relay-finalized deterministic projection,
///     NOT raw input trace. It includes NoOp injection for missing players.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TickCommands {
    pub tick: u32,
    /// Commands sorted by (player_id, sort_tag). Deterministic ordering.
    pub commands: Vec<GameCommand>,
}

// ═══════════════════════════════════════════════════════════════
// BroadcastFrame — transport envelope (relay → client)
// ═══════════════════════════════════════════════════════════════

/// Relay-to-client broadcast frame for one finalized tick.
///
/// BroadcastFrame is a **transport envelope** — it wraps TickCommands
/// with network metadata. The payload (TickCommands) can be extracted
/// and stored independently (for replay, reconnect, etc.).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BroadcastFrame {
    pub game_id: u64,
    pub ruleset_version: u32,
    pub payload: TickCommands,
    /// Relay wall clock timestamp (debug only, not used for determinism).
    pub relay_ts_ms: u64,
}

// ═══════════════════════════════════════════════════════════════
// PlayerTickFrame — client → relay
// ═══════════════════════════════════════════════════════════════

/// Client-to-relay upstream frame containing one player's commands for one tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerTickFrame {
    pub magic: u16,          // Protocol magic number for validation
    pub game_id: u64,
    pub tick: u32,           // Target tick (current_tick + input_delay)
    pub player_id: u8,
    pub commands: Vec<GameCommand>,
    /// Client-side monotonically increasing sequence number (idempotency).
    pub player_sid: u64,
}

// ═══════════════════════════════════════════════════════════════
// Reconnect — request / response
// ═══════════════════════════════════════════════════════════════

/// Client-to-relay reconnect request after disconnection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconnectRequest {
    pub game_id: u64,
    /// The last tick this client successfully consumed before disconnect.
    pub last_tick_consumed: u32,
}

/// Relay-to-client reconnect response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconnectResponse {
    pub game_id: u64,
    pub ruleset_version: u32,
    pub seed: u64,
    pub map_spec_hash: u64,
    /// First tick of the command log (= last_tick_consumed + 1).
    pub first_tick: u32,
    /// Command log from first_tick to current finalized tick.
    pub ticks: Vec<TickCommands>,
    /// All player states at the time of reconnect.
    pub players: Vec<PlayerState>,
}

// ═══════════════════════════════════════════════════════════════
// Top-level game messages (client ↔ relay)
// ═══════════════════════════════════════════════════════════════

/// Messages sent from client to relay.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RelayClientMessage {
    /// Join a new game.
    JoinGame(GameInitParams),
    /// Submit input commands for a tick.
    PlayerTick(PlayerTickFrame),
    /// Reconnect after disconnect.
    Reconnect(ReconnectRequest),
}

/// Messages sent from relay to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RelayServerMessage {
    /// Game session accepted.
    GameJoined { game_id: u64, player_id: u8 },
    /// Reconnect response with full state recovery data.
    ReconnectResponse(ReconnectResponse),
    /// Broadcast frame for a finalized tick.
    Broadcast(BroadcastFrame),
    /// Game over notification.
    GameOver { reason: String },
    /// Error / version mismatch.
    Error { code: u32, message: String },
}

// ═══════════════════════════════════════════════════════════════
// NetworkCommandSource — 网络模式命令源
// ═══════════════════════════════════════════════════════════════

use std::collections::HashMap;
use crate::driver::DriverContext;

/// Network-mode command source: consumes relay-finalized CommandBatches.
///
/// **No merge logic.** `commands_for_tick()` returns only what the relay broadcast.
/// Local player input is uplinked via `cmd_buf` → relay → broadcast back to all clients.
/// This source does NOT read from the Bevy `cmd_buf` at execution time.
#[derive(Default)]
pub struct NetworkCommandSource {
    /// Incoming finalized command batches from relay, indexed by tick.
    pub relay_buffer: HashMap<u32, TickCommands>,
    /// Client-side connection state.
    pub game_id: u64,
    pub player_id: u8,
    pub ruleset_version: u32,
    pub connected: bool,
}

impl NetworkCommandSource {
    /// Check whether this source has received a finalized batch for the given tick.
    pub fn is_tick_ready(&self, tick: u32) -> bool {
        self.relay_buffer.contains_key(&tick)
    }

    /// Consume commands for a tick from the relay buffer.
    ///
    /// **Only reads from relay_buffer.** The `ctx` parameter is IGNORED —
    /// the Bevy `cmd_buf` is NOT read by this source in network mode.
    pub fn commands_for_tick(&mut self, tick: u32, _ctx: &DriverContext) -> Vec<GameCommand> {
        self.relay_buffer
            .remove(&tick)
            .map(|batch| batch.commands)
            .unwrap_or_default()
    }

    /// Network mode always produces a replay.
    pub fn should_record(&self) -> bool {
        true
    }

    /// Accept a broadcast frame from the relay and store it.
    pub fn push_broadcast(&mut self, frame: BroadcastFrame) {
        self.relay_buffer.insert(frame.payload.tick, frame.payload);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::command::{Action, CommandBuffer, GameCommand};

    /// Helper: create a minimal TickCommands.
    fn make_tick(tick: u32, player_id: u8) -> TickCommands {
        TickCommands {
            tick,
            commands: vec![GameCommand {
                tick,
                player_id,
                action: Action::NoOp,
            }],
        }
    }

    #[test]
    fn test_is_tick_ready_false_when_empty() {
        let source = NetworkCommandSource::default();
        assert!(!source.is_tick_ready(100));
    }

    #[test]
    fn test_is_tick_ready_true_after_push() {
        let mut source = NetworkCommandSource::default();
        source.push_broadcast(BroadcastFrame {
            game_id: 1,
            ruleset_version: 1,
            payload: make_tick(100, 0),
            relay_ts_ms: 0,
        });
        assert!(source.is_tick_ready(100));
    }

    #[test]
    fn test_commands_for_tick_ignores_ctx() {
        let mut source = NetworkCommandSource::default();
        source.push_broadcast(BroadcastFrame {
            game_id: 1,
            ruleset_version: 1,
            payload: make_tick(100, 0),
            relay_ts_ms: 0,
        });

        // Even if ctx points to a cmd_buf with commands, should only return relay batch
        let ctx = DriverContext {
            bevy_cmds: &CommandBuffer(vec![GameCommand {
                tick: 100,
                player_id: 99,
                action: Action::NoOp,
            }]),
        };

        let cmds = source.commands_for_tick(100, &ctx);
        assert_eq!(cmds.len(), 1);
        // ctx content ignored — returned command has player_id from relay batch
        assert_eq!(cmds[0].player_id, 0);
    }

    #[test]
    fn test_commands_for_tick_returns_nothing_for_missing_tick() {
        let mut source = NetworkCommandSource::default();
        let ctx = DriverContext {
            bevy_cmds: &CommandBuffer(Vec::new()),
        };
        let cmds = source.commands_for_tick(999, &ctx);
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_should_record_always_true() {
        let source = NetworkCommandSource::default();
        assert!(source.should_record());
    }

    #[test]
    fn test_push_broadcast_then_remove_consumes_once() {
        let mut source = NetworkCommandSource::default();
        source.push_broadcast(BroadcastFrame {
            game_id: 1,
            ruleset_version: 1,
            payload: make_tick(50, 1),
            relay_ts_ms: 0,
        });

        // First call consumes
        let ctx = DriverContext {
            bevy_cmds: &CommandBuffer(Vec::new()),
        };
        let cmds1 = source.commands_for_tick(50, &ctx);
        assert_eq!(cmds1.len(), 1);

        // Second call returns empty (already consumed)
        let cmds2 = source.commands_for_tick(50, &ctx);
        assert!(cmds2.is_empty());
    }
}

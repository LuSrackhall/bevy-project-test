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

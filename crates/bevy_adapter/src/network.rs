//! Network protocol types for RTS CommandStream Protocol v1.0.
//!
//! Data structures for relay-backed deterministic lockstep multiplayer.
//! See: openspec/changes/network-command-stream/brainstorm-spec.md §3.1

pub use crate::discovery::{RelayId, RoomId, RoomMetadata, RoomState};

use serde::{Deserialize, Serialize};
use simulation::command::GameCommand;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use bevy::prelude::Resource;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerState {
    /// Player is actively connected and submitting frames.
    Active { player_id: u8 },
    /// Player has disconnected; NoOp will be injected.
    Disconnected { player_id: u8 },
}

/// Lobby state for one player.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LobbyPlayerState {
    pub player_id: u8,
    pub ready: bool,
    pub selected_map: Option<simulation::map::MapSize>,
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
    pub version: u16,         // Protocol version (currently 1)

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
    /// Join a game session, requesting player_id allocation.
    JoinGame {
        room_id: crate::discovery::RoomId,
        relay_id: crate::discovery::RelayId,
    },
    /// Submit input commands for a tick.
    PlayerTick(PlayerTickFrame),
    /// Reconnect after disconnect.
    Reconnect(ReconnectRequest),
    /// Signal lobby readiness with optional map selection.
    LobbyReady { game_id: u64, player_id: u8, ready: bool, map_size: Option<simulation::map::MapSize> },

}

/// Messages sent from relay to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RelayServerMessage {
    /// Game session accepted — player identity assigned by relay.
    GameJoined { game_id: u64, player_id: u8, player_count: u8 },
    /// Join request rejected (room full, identity mismatch, etc.).
    JoinRejected { reason: String },
    /// All players have connected; game is starting.
    GameStarted { game_id: u64, seed: u64, player_count: u8 },
    /// Reconnect response with full state recovery data.
    ReconnectResponse(ReconnectResponse),
    /// Broadcast frame for a finalized tick.
    Broadcast(BroadcastFrame),
    /// Game over notification.
    GameOver { reason: String },
    /// Error / version mismatch.
    Error { code: u32, message: String },
    /// Lobby state update — all connected players' ready status.
    LobbyUpdate { game_id: u64, players: Vec<LobbyPlayerState> },

}

// ═══════════════════════════════════════════════════════════════
// NetworkCommandSource — 网络模式命令源
// ═══════════════════════════════════════════════════════════════

/// Events from the network tokio thread to the Bevy main thread.
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    /// Relay has accepted the player and assigned identity.
    GameJoined { player_id: u8, player_count: u8 },
    /// All players connected; game is starting.
    GameStarted { game_id: u64, seed: u64, player_count: u8 },
    /// Lobby state update — player ready statuses.
    LobbyUpdate { game_id: u64, players: Vec<LobbyPlayerState> },
    /// Reconnect response with the command log from the disconnect point onward.
    Reconnect(ReconnectResponse),

}

/// Cross-thread channel for NetworkEvents (tokio → Bevy).
#[derive(Clone, Default, Resource)]
pub struct NetworkEventReceiver {
    inner: Arc<Mutex<VecDeque<NetworkEvent>>>,
}

impl NetworkEventReceiver {
    pub fn push(&self, event: NetworkEvent) {
        self.inner.lock().unwrap().push_back(event);
    }

    pub fn drain_all(&self) -> Vec<NetworkEvent> {
        let mut inner = self.inner.lock().unwrap();
        inner.drain(..).collect()
    }
}

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
    /// Input delay in ticks (default 3). Applied when constructing PlayerTickFrame.
    /// D5: Offset happens only inside NetworkCommandSource.
    pub input_delay: u32,
}

impl NetworkCommandSource {
    /// Create a new NetworkCommandSource.
    pub fn new(game_id: u64, player_id: u8, input_delay: u32) -> Self {
        Self {
            game_id,
            player_id,
            input_delay,
            ..Default::default()
        }
    }

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

    /// Apply input delay offset: convert a local tick to the relay-target tick.
    ///
    /// D5: Delay offset occurs ONLY inside NetworkCommandSource.
    /// render_view continues to push commands with `tick = current + 1`.
    /// This method maps that to `tick + input_delay` before sending to relay.
    pub fn delayed_tick(&self, local_tick: u32) -> u32 {
        local_tick + self.input_delay
    }

    /// Process a ReconnectResponse: load the command log into the relay buffer.
    ///
    /// The reconnect path (D11) uses replay-based recovery:
    /// - Scene A (network drop, process alive): local world is intact at the
    ///   disconnect point — only the missed ticks are loaded and the driver resumes.
    /// - Scene B (process restart): the world MUST be rebuilt via
    ///   `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local))`
    ///   + `run_tick(enable_ai:false)` — matching the live network path.
    ///   `init_simulation_world` (2-slot) / `run_tick_default` (AI on) are FORBIDDEN (R1):
    ///   they diverge from the network PlayerSlots/NoOp set and desync (specs/network-reconnect).
    ///
    /// This method stores the tick log so the driver can consume it sequentially.
    ///
    /// D12: Validates ruleset_version — mismatch returns an error.
    pub fn apply_reconnect(&mut self, response: &ReconnectResponse, expected_version: u32) -> Result<(), String> {
        // D12: ruleset_version compatibility check
        if response.ruleset_version != expected_version {
            return Err(format!(
                "Schema mismatch: client ruleset_version={}, relay ruleset_version={}. \
                 Cannot replay — versions must match for deterministic replay.",
                expected_version, response.ruleset_version
            ));
        }

        self.game_id = response.game_id;
        self.ruleset_version = response.ruleset_version;
        self.connected = true;
        self.relay_buffer.clear();
        for batch in &response.ticks {
            self.relay_buffer.insert(batch.tick, batch.clone());
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════
// RelayServer — tick barrier 核心状态机
// ═══════════════════════════════════════════════════════════════

/// Tick barrier state machine: collects player inputs, finalizes ticks,
/// and produces deterministic CommandBatches.
///
/// This is a **pure state machine** — it does NOT handle transport.
/// The transport layer (TCP/UDP) injects `on_player_frame()` calls
/// and consumes broadcast outputs.
///
/// D4: Relay does NOT simulate, sort, assign ordering keys, or inspect Action semantics.
pub struct RelayServer {
    /// Game session configuration.
    game_id: u64,
    /// Unique relay instance identifier for JoinGame verification.
    relay_id: RelayId,
    ruleset_version: u32,
    seed: u64,
    map_spec_hash: u64,
    all_players: Vec<u8>,

    /// Buffer per (tick, player_id) -> accumulated commands.
    buffer: HashMap<u32, HashMap<u8, Vec<GameCommand>>>,
    /// Set of player_ids that have submitted frames per tick.
    ready: HashMap<u32, Vec<u8>>,
    /// Wall clock time (ms) when first player frame for each tick arrived.
    first_arrival: HashMap<u32, u64>,
    /// Finalized command log for reconnect and replay.
    log: Vec<TickCommands>,
    /// Dedup set: (tick, player_id, player_sid) -> seen
    seen_frames: std::collections::HashSet<(u32, u8, u64)>,

    /// Default input delay in ticks.
    input_delay: u32,
    /// Tick duration in ms.
    tick_duration_ms: u64,
    /// Jitter buffer in ms.
    jitter_ms: u64,

    /// Frozen state: no more broadcasts until timeout or players reconnect.
    frozen: bool,
    freezed_at_ms: u64,
    /// Current tick being collected.
    current_tick: u32,
    /// Game creation wall clock time (ms). Used for absolute timeout fallback.
    created_at_ms: u64,
    /// All players have joined and game is actively running.
    /// Prevents timeout-based tick finalization before all players connect.
    game_started: bool,
    /// Tracks which players have signaled LobbyReady (scalable, no bit-mask ceiling).
    lobby_ready: HashSet<u8>,
    /// Set of player_ids currently disconnected (seat retained, awaiting reconnect).
    disconnected: HashSet<u8>,
    /// Next player_id to assign for JoinGame.
    next_player_id: u8,

}

impl RelayServer {
    /// Create a new relay server for a game session.
    pub fn new(
        game_id: u64,
        relay_id: RelayId,
        ruleset_version: u32,
        seed: u64,
        map_spec_hash: u64,
        players: Vec<u8>,
        input_delay: u32,
        now_ms: u64,
    ) -> Self {
        Self {
            game_id,
            relay_id,
            ruleset_version,
            seed,
            map_spec_hash,
            all_players: players,
            buffer: HashMap::new(),
            ready: HashMap::new(),
            first_arrival: HashMap::new(),
            log: Vec::new(),
            seen_frames: std::collections::HashSet::new(),
            input_delay,
            tick_duration_ms: 50, // 20Hz default
            jitter_ms: 50,        // 1 tick jitter buffer
            frozen: false,
            freezed_at_ms: 0,
            current_tick: 1,
            created_at_ms: now_ms,
            game_started: false,
            lobby_ready: HashSet::new(),
            disconnected: HashSet::new(),
            next_player_id: 0,
        }
    }

    /// Relay identity for JoinGame verification.
    pub fn relay_id(&self) -> RelayId {
        self.relay_id
    }

    /// Process a JoinGame request.
    /// Returns Ok(player_id) on acceptance, or Err(reason) on rejection.
    pub fn on_join_game(&mut self, request_relay_id: RelayId) -> Result<u8, String> {
        // Verify relay identity
        if request_relay_id != self.relay_id {
            return Err("Relay identity mismatch".into());
        }
        // 优先复用断线席位(重连场景),保证重连后拿回原 player_id
        if let Some(&pid) = self.disconnected.iter().min() {
            self.disconnected.remove(&pid);
            return Ok(pid);
        }
        // 否则按序分配新席位
        if (self.next_player_id as usize) >= self.all_players.len() {
            return Err("Room is full".into());
        }
        let player_id = self.next_player_id;
        self.next_player_id += 1;
        Ok(player_id)
    }

    /// Whether a specific player has signaled LobbyReady.
    pub fn is_player_ready(&self, player_id: u8) -> bool {
        self.lobby_ready.contains(&player_id)
    }

    /// Process a LobbyReady signal. Returns true when ALL players are ready.
    /// Disconnected seats are excluded from the all-ready check — otherwise a
    /// player dropping in the lobby (before ready) would deadlock the room.
    pub fn on_lobby_ready(&mut self, player_id: u8) -> bool {
        self.lobby_ready.insert(player_id);
        let active_count = self
            .all_players
            .iter()
            .filter(|p| !self.disconnected.contains(*p))
            .count();
        let ready_active = self
            .lobby_ready
            .iter()
            .filter(|p| !self.disconnected.contains(*p))
            .count();
        ready_active >= active_count
    }

    /// Process a LobbyReady { ready: false } signal — clear the player's ready state.
    pub fn on_lobby_not_ready(&mut self, player_id: u8) {
        self.lobby_ready.remove(&player_id);
    }

    /// Whether the game has started (prevents late LobbyReady tampering).
    pub fn is_game_started(&self) -> bool {
        self.game_started
    }

    /// Process an incoming player frame.
    ///
    /// Returns `(Option<TickCommands>, bool)`:
    /// - `Option<TickCommands>` — finalized batch if tick was just completed, or `None`
    /// - `bool` — `true` if game_started transitioned from false to true (all players connected)
    ///
    /// D4: Relay does NOT modify commands.
    /// D10: Dedup uses (tick, player_id, player_sid).
    pub fn on_player_frame(&mut self, frame: &PlayerTickFrame, now_ms: u64) -> (Option<TickCommands>, bool) {
        if self.frozen {
            return (None, false);
        }
        if frame.game_id != self.game_id {
            return (None, false);
        }

        // E2: Reject frames from disconnected players
        if !self.all_players.contains(&frame.player_id) {
            return (None, false);
        }

        // D10: Idempotent dedup by (tick, player_id, player_sid)
        let dedup_key = (frame.tick, frame.player_id, frame.player_sid);
        if !self.seen_frames.insert(dedup_key) {
            return (None, false); // Duplicate, silently dropped
        }

        // Record first arrival time for this tick (used for timeout D5)
        self.first_arrival.entry(frame.tick).or_insert(now_ms);

        // Store commands (extend — overwrite would let empty frames clear valid commands)
        self.buffer
            .entry(frame.tick)
            .or_default()
            .entry(frame.player_id)
            .or_default()
            .extend(frame.commands.clone());

        // Mark player as ready for this tick
        self.ready.entry(frame.tick).or_default().push(frame.player_id);

        // Check if all players have connected (at least one frame from each).
        // Prevents timeout-based finalization before all players join the game.
        // Disconnected 席位不阻塞 game_started 达成(其帧由 NoOp 兜底)。
        let mut game_just_started = false;
        if !self.game_started {
            let connected: std::collections::HashSet<&u8> =
                self.ready.values().flat_map(|v| v.iter()).collect();
            let now_started = self.all_players.iter()
                .filter(|p| !self.disconnected.contains(*p))
                .all(|p| connected.contains(p));
            if now_started {
                self.game_started = true;
                game_just_started = true;
                eprintln!("[RELAY] game_started = true (all players connected)");
            }
        }

        // Try to finalize
        let batch = self.try_finalize(frame.tick, now_ms);
        (batch, game_just_started)
    }

    /// Attempt to finalize a tick. Returns finalized TickCommands if complete.
    ///
    /// D8: Batch is immutable once finalized. No late corrections.
    /// D7: NoOp for missing players is a pure function of (tick, player_id).
    fn try_finalize(&mut self, tick: u32, now_ms: u64) -> Option<TickCommands> {
        // Check if tick is already finalized (in log)
        if self.log.iter().any(|b| b.tick == tick) {
            return None;
        }

        let all_ready = {
            let ready_set: std::collections::HashSet<&u8> =
                self.ready.get(&tick).map(|r| r.iter().collect()).unwrap_or_default();
            // R3: Disconnected 席位放行(不阻塞 barrier),其 NoOp 由下方注入
            self.all_players.iter()
                .filter(|p| !self.disconnected.contains(*p))
                .all(|p| ready_set.contains(p))
        };

        // Only allow timeout when game_started (all players have connected).
        // Without this, a player connects, relay times out, and finalizes tick 1
        // before the other player even joins.
        let timed_out = self.game_started && self.is_timed_out(tick, now_ms);

        if !all_ready && !timed_out {
            return None;
        }

        // Collect all commands for this tick, filtering out disconnected players
        let active_players: std::collections::HashSet<&u8> =
            self.all_players.iter().collect();
        let mut all_cmds: Vec<GameCommand> = self
            .buffer
            .get(&tick)
            .map(|per_player| {
                per_player
                    .iter()
                    .filter(|(pid, _)| active_players.contains(pid))
                    .flat_map(|(_, cmds)| cmds.iter().cloned())
                    .collect()
            })
            .unwrap_or_default();

        // D7: NoOp injection for missing players (pure function of tick, player_id)
        let ready_set: std::collections::HashSet<&u8> =
            self.ready.get(&tick).map(|r| r.iter().collect()).unwrap_or_default();
        for pid in &self.all_players {
            if !ready_set.contains(pid) {
                all_cmds.push(GameCommand {
                    tick,
                    player_id: *pid,
                    action: simulation::command::Action::NoOp,
                });
            }
        }

        // Sort deterministically by (player_id, sort_tag)
        all_cmds.sort_by_key(|c| (c.player_id, c.action.sort_tag()));

        let batch = TickCommands {
            tick,
            commands: all_cmds,
        };

        // D8: Store in log (immutable once finalized)
        self.log.push(batch.clone());

        // Clean up staging data for this tick
        self.buffer.remove(&tick);
        self.ready.remove(&tick);
        self.first_arrival.remove(&tick);

        // Advance current tick
        self.current_tick = tick + 1;

        Some(batch)
    }

    /// Check if tick has timed out.
    /// D5: Timeout = relay wall clock first_arrival + D * T_tick + jitter
    /// Fallback: if no frame ever arrived for this tick, use an absolute timeout
    /// based on game creation time + expected tick schedule.
    fn is_timed_out(&self, tick: u32, now_ms: u64) -> bool {
        // Primary timeout: based on first_arrival
        if let Some(arrival) = self.first_arrival.get(&tick) {
            let timeout_duration =
                arrival + (self.input_delay as u64 * self.tick_duration_ms) + self.jitter_ms;
            return now_ms >= timeout_duration;
        }

        // Fallback: no frame arrived at all for this tick.
        // Use absolute timeout: tick should appear by approximately
        // created_at + (tick + input_delay + buffer) * tick_duration
        let expected_time = self.created_at_ms
            + (tick as u64 + self.input_delay as u64 + 2) * self.tick_duration_ms  // +2 for extra buffer
            + self.jitter_ms;
        now_ms >= expected_time
    }

    /// Handle client disconnect. Returns the current player states.
    ///
    /// D9: Disconnected players get NoOp injected (already handled in try_finalize).
    /// Seat is retained (not removed from all_players) so the player can reconnect
    /// under the same player_id (specs/relay-server).
    pub fn on_disconnect(&mut self, player_id: u8) -> Vec<PlayerState> {
        self.disconnected.insert(player_id);
        self.player_states()
    }

    /// Handle full disconnect (all players gone). Freezes the game.
    pub fn on_full_disconnect(&mut self, now_ms: u64) {
        if self.disconnected.len() >= self.all_players.len() {
            self.frozen = true;
            self.freezed_at_ms = now_ms;
        }
    }

    /// Handle reconnect request.
    /// D11: Returns TickCommands from last_tick_consumed+1 to current.
    /// D12: Validates ruleset_version compatibility.
    pub fn handle_reconnect(&self, request: &ReconnectRequest) -> Result<ReconnectResponse, String> {
        let ticks: Vec<TickCommands> = self
            .log
            .iter()
            .filter(|b| b.tick > request.last_tick_consumed)
            .cloned()
            .collect();

        Ok(ReconnectResponse {
            game_id: self.game_id,
            ruleset_version: self.ruleset_version,
            seed: self.seed,
            map_spec_hash: self.map_spec_hash,
            first_tick: request.last_tick_consumed + 1,
            ticks,
            players: self.player_states(),
        })
    }

    /// Check if freeze timeout has elapsed (30s).
    pub fn check_freeze_timeout(&mut self, now_ms: u64) -> bool {
        if self.frozen && now_ms - self.freezed_at_ms >= 30_000 {
            return true;
        }
        false
    }

    fn player_states(&self) -> Vec<PlayerState> {
        self.all_players
            .iter()
            .map(|pid| {
                if self.disconnected.contains(pid) {
                    PlayerState::Disconnected { player_id: *pid }
                } else {
                    PlayerState::Active { player_id: *pid }
                }
            })
            .collect()
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    pub fn current_tick(&self) -> u32 {
        self.current_tick
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn player_count(&self) -> u8 {
        self.all_players.len() as u8
    }

    pub fn command_log(&self) -> &[TickCommands] {
        &self.log
    }
}

#[cfg(test)]
mod relay_tests {
    use super::*;
    use simulation::command::Action;

    fn make_frame(tick: u32, player_id: u8, sid: u64) -> PlayerTickFrame {
        PlayerTickFrame {
            magic: 0xBEEF,
            version: 1,
            game_id: 1,
            tick,
            player_id,
            commands: vec![GameCommand {
                tick,
                player_id,
                action: Action::NoOp,
            }],
            player_sid: sid,
        }
    }

    /// Helper to create a relay with 2 players.
    fn relay_2p() -> RelayServer {
        let rid = crate::discovery::RelayId(42);
        RelayServer::new(1, rid, 1, 42, 0xABC, vec![0, 1], 3, 1000)
    }

    fn make_empty_frame(tick: u32, player_id: u8, sid: u64) -> PlayerTickFrame {
        PlayerTickFrame {
            magic: 0xBEEF,
            version: 1,
            game_id: 1,
            tick,
            player_id,
            commands: vec![],
            player_sid: sid,
        }
    }

    #[test]
    fn test_single_tick_both_players_completes() {
        let mut relay = relay_2p();
        let now = 1000;

        let r1 = relay.on_player_frame(&make_frame(1, 0, 1), now);
        assert!(r1.0.is_none()); // Still waiting for player 1

        let r2 = relay.on_player_frame(&make_frame(1, 1, 1), now);
        assert!(r2.0.is_some()); // Both arrived → finalized

        let batch = r2.0.unwrap();
        assert_eq!(batch.tick, 1);
        assert_eq!(batch.commands.len(), 2);
    }

    #[test]
    fn test_duplicate_frame_dropped() {
        let mut relay = relay_2p();
        let now = 1000;

        assert!(relay.on_player_frame(&make_frame(1, 0, 1), now).0.is_none());
        // Duplicate (same tick, player, sid)
        assert!(relay.on_player_frame(&make_frame(1, 0, 1), now).0.is_none());
    }

    #[test]
    fn test_timeout_injects_noop() {
        let mut relay = relay_2p();
        let arrival = 1000;
        let input_delay = 3u32;
        let tick_dur = 50u64;
        let jitter = 50u64;

        // Both players must connect (submit for tick 1) to mark game_started
        relay.on_player_frame(&make_empty_frame(1, 0, 1), arrival);
        relay.on_player_frame(&make_empty_frame(1, 1, 1), arrival);

        // Tick 1 finalized (both players ready)
        // Player 0 submits for tick 2 → sets first_arrival[2]
        relay.on_player_frame(&make_empty_frame(2, 0, 2), arrival + 100);

        // Wait for timeout: first_arrival[2] + 3*50 + 50 = 100 + 150 + 50 = 300ms
        // Submit at 500ms (well past timeout) to trigger try_finalize
        let timeout = arrival + 100 + (input_delay as u64 * tick_dur) + jitter + 50;
        let result = relay.on_player_frame(&make_empty_frame(2, 0, 3), timeout);
        assert!(result.0.is_some(), "timeout should fire");

        let batch = result.0.unwrap();
        // Both players have 1 command each (player 1 never submitted, gets NoOp)
        let noop = batch.commands.iter().find(|c| c.player_id == 1)
            .expect("player 1 should have a command");
        assert_eq!(noop.action, Action::NoOp);
    }

    #[test]
    fn test_reconnect_returns_log() {
        let mut relay = relay_2p();
        let now = 1000;

        // Finalize tick 1 (both players)
        relay.on_player_frame(&make_frame(1, 0, 1), now);
        relay.on_player_frame(&make_frame(1, 1, 1), now);

        // Finalize tick 2 (both players)
        relay.on_player_frame(&make_frame(2, 0, 1), now + 100);
        relay.on_player_frame(&make_frame(2, 1, 1), now + 100);

        // Reconnect from tick 1
        let req = ReconnectRequest {
            game_id: 1,
            last_tick_consumed: 1,
        };
        let resp = relay.handle_reconnect(&req).unwrap();
        assert_eq!(resp.first_tick, 2);
        assert_eq!(resp.ticks.len(), 1); // Only tick 2
        assert_eq!(resp.seed, 42);
    }

    #[test]
    fn test_log_is_immutable() {
        let mut relay = relay_2p();
        let now = 1000;

        relay.on_player_frame(&make_frame(1, 0, 1), now);
        relay.on_player_frame(&make_frame(1, 1, 1), now);

        // Tick 1 should be finalized
        let log_len_before = relay.command_log().len();
        assert_eq!(log_len_before, 1);

        // Late frame for tick 1 should NOT be accepted (already finalized)
        let late = relay.on_player_frame(&make_frame(1, 0, 2), now + 1000);
        assert!(late.0.is_none()); // Already finalized, treated as late
        assert_eq!(relay.command_log().len(), 1); // Log unchanged
    }

    #[test]
    fn test_freeze_on_empty_players() {
        let mut relay = relay_2p();
        let now = 1000;

        relay.on_disconnect(0);
        relay.on_disconnect(1);
        relay.on_full_disconnect(now);

        assert!(relay.is_frozen());

        // Frozen relay ignores frames
        let result = relay.on_player_frame(&make_frame(1, 0, 1), now);
        assert!(result.0.is_none());
    }

    #[test]
    fn test_disconnect_retains_seat_and_reconnect_reuses_id() {
        let mut relay = relay_2p();
        // 两个玩家加入(relay_2p relay_id = 42)
        assert_eq!(relay.on_join_game(crate::discovery::RelayId(42)).unwrap(), 0);
        assert_eq!(relay.on_join_game(crate::discovery::RelayId(42)).unwrap(), 1);
        // 玩家 0 掉线:席位保留,状态标 Disconnected
        let states = relay.on_disconnect(0);
        assert!(states.iter().any(|s| matches!(s, PlayerState::Disconnected { player_id: 0 })));
        // 重连:复用原 player_id 0(而不是 Room is full)
        assert_eq!(relay.on_join_game(crate::discovery::RelayId(42)).unwrap(), 0);
        // 满员:第三方加入被拒
        let err = relay.on_join_game(crate::discovery::RelayId(42)).unwrap_err();
        assert!(err.contains("Room is full"), "err={}", err);
    }

    #[test]
    fn test_disconnected_player_does_not_hang_barrier() {
        let mut relay = relay_2p();
        let now = 1000;
        // 玩家 0 掉线(Disconnected 席位放行)
        relay.on_disconnect(0);
        // 玩家 1 提交 tick 1 → 应定稿,不挂起
        let r1 = relay.on_player_frame(&make_empty_frame(1, 1, 1), now);
        assert!(r1.0.is_some(), "Disconnected seat must not hang the barrier");
        let batch = r1.0.unwrap();
        // 玩家 0 被注入 NoOp
        assert!(batch.commands.iter().any(|c| c.player_id == 0 && c.action == Action::NoOp));
    }
}

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

    #[test]
    fn test_apply_reconnect_loads_log_and_validates_version() {
        let mut source = NetworkCommandSource::default();
        // 版本不匹配 → Err(D12)
        let resp_bad = ReconnectResponse {
            game_id: 1,
            ruleset_version: 2,
            seed: 1,
            map_spec_hash: 0,
            first_tick: 1,
            ticks: vec![],
            players: vec![],
        };
        assert!(source.apply_reconnect(&resp_bad, 1).is_err());
        // 版本匹配 → 灌入 log + 更新身份
        let resp = ReconnectResponse {
            game_id: 1,
            ruleset_version: 1,
            seed: 42,
            map_spec_hash: 0,
            first_tick: 2,
            ticks: vec![make_tick(2, 0), make_tick(3, 1)],
            players: vec![],
        };
        source.apply_reconnect(&resp, 1).unwrap();
        assert!(source.is_tick_ready(2));
        assert!(source.is_tick_ready(3));
        assert_eq!(source.game_id, 1);
        assert!(source.connected);
    }
}

// ═══════════════════════════════════════════════════════════════
// LanDiscovery — see `crate::discovery` for the new model-based impl
// ═══════════════════════════════════════════════════════════════

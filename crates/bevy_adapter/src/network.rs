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
// Reconnect — request / response / pages
// ═══════════════════════════════════════════════════════════════

/// Ticks per reconnect page. A page (~3.2 KB for ~100B/tick) maps to a few
/// MTU fragments, keeping per-message delivery bounded and progressive.
pub const PAGE_TICKS: u32 = 32;

/// Client-to-relay reconnect request after disconnection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconnectRequest {
    pub game_id: u64,
    /// The last tick this client successfully consumed before disconnect.
    pub last_tick_consumed: u32,
}

/// Relay-to-client reconnect metadata (page_count pages follow on the
/// reliable Control channel). Carries no command log — pages do.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconnectResponse {
    pub game_id: u64,
    pub ruleset_version: u32,
    pub seed: u64,
    pub map_spec_hash: u64,
    /// Map size for Scene B world rebuild (must match the live clients' map).
    pub map_size: simulation::map::MapSize,
    /// First tick of the command log (= last_tick_consumed + 1).
    pub first_tick: u32,
    /// Number of log entries with tick > last_tick_consumed (count, not span).
    pub total_ticks: u32,
    /// ceil(total_ticks / PAGE_TICKS); 0 when total_ticks = 0.
    pub page_count: u32,
    /// All player states at the time of reconnect.
    pub players: Vec<PlayerState>,
}

/// One page of the reconnect command log, pushed after the metadata response.
/// Pages cover contiguous tick ranges bucketed by tick VALUE (the finalized
/// log is append-ordered, not tick-ordered — see D2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconnectPage {
    pub page_index: u32,
    pub page_count: u32,
    /// First tick of this page (for defensive validation).
    pub first_tick: u32,
    pub ticks: Vec<TickCommands>,
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
    /// All players have connected; game is starting. Carries the map size so
    /// clients build a world matching the game (no hardcoded default).
    GameStarted { game_id: u64, seed: u64, player_count: u8, map_size: simulation::map::MapSize },
    /// Reconnect response with metadata; page_count pages follow on Control.
    ReconnectResponse(ReconnectResponse),
    /// One page of the reconnect command log (progressive replay).
    ReconnectPage(ReconnectPage),
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
    GameStarted { game_id: u64, seed: u64, player_count: u8, map_size: simulation::map::MapSize },
    /// Lobby state update — player ready statuses.
    LobbyUpdate { game_id: u64, players: Vec<LobbyPlayerState> },
    /// Reconnect response with the reconnect metadata (first_tick, total_ticks, page_count).
    Reconnect(ReconnectResponse),
    /// One page of the reconnect command log.
    ReconnectPage(ReconnectPage),

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
/// Reconnect replay cursor — set when reconnect metadata is applied, cleared
/// when the last page is applied. Drives `apply_reconnect_page` validation.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReconnectCursor {
    pub first_tick: u32,
    pub total_ticks: u32,
    pub page_count: u32,
    pub next_page: u32,
}

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
    /// Reconnect page cursor (Some between metadata and last page).
    pub reconnect_meta: Option<ReconnectCursor>,
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

    /// Process a ReconnectResponse (metadata): validate, set the replay cursor,
    /// and clear only stale ticks. Reconnect log pages are applied progressively
    /// via `apply_reconnect_page` as they arrive on the Control channel.
    ///
    /// D3: `relay_buffer` is cleared ONLY below `first_tick` — live broadcasts
    /// for ticks finalized after the reconnect request (≥ first_tick, outside the
    /// page range) must survive, else a deterministic gap forms.
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
        self.relay_buffer.retain(|tick, _| *tick >= response.first_tick);
        // page_count = 0 → nothing to fetch; replay completes immediately.
        self.reconnect_meta = (response.page_count > 0).then(|| ReconnectCursor {
            first_tick: response.first_tick,
            total_ticks: response.total_ticks,
            page_count: response.page_count,
            next_page: 0,
        });
        Ok(())
    }

    /// Apply one reconnect page, inserting its ticks into the relay buffer.
    ///
    /// D4: rejects out-of-order/duplicate/out-of-range pages and pages whose
    /// `page_count` mismatches the metadata (defense against stale pages from a
    /// previous session). The reliable Control channel already delivers in order,
    /// so a rejection means a protocol/state error — surfaced for logging.
    pub fn apply_reconnect_page(&mut self, page: &ReconnectPage) -> Result<(), String> {
        let meta = match self.reconnect_meta {
            Some(m) => m,
            None => return Err("ReconnectPage received without reconnect metadata".into()),
        };
        if page.page_count != meta.page_count {
            return Err(format!(
                "page_count mismatch: page={}, metadata={}",
                page.page_count, meta.page_count
            ));
        }
        if page.page_index != meta.next_page {
            return Err(format!(
                "out-of-order page: got {}, expected {}",
                page.page_index, meta.next_page
            ));
        }
        if page.page_index >= meta.page_count {
            return Err(format!(
                "page_index {} out of range (page_count={})",
                page.page_index, meta.page_count
            ));
        }
        for batch in &page.ticks {
            if batch.tick < meta.first_tick {
                return Err(format!(
                    "page contains tick {} below first_tick {}",
                    batch.tick, meta.first_tick
                ));
            }
            self.relay_buffer.insert(batch.tick, batch.clone());
        }
        let next_page = meta.next_page + 1;
        if next_page >= meta.page_count {
            self.reconnect_meta = None; // last page — replay complete
        } else {
            self.reconnect_meta = Some(ReconnectCursor { next_page, ..meta });
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
    /// Map size for the game (Scene B rebuild + map spec).
    map_size: simulation::map::MapSize,
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
        map_size: simulation::map::MapSize,
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
            map_size,
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
    /// Returns `Ok((player_id, reused_disconnected_seat))` on acceptance (the
    /// bool signals a reconnect — the seat was previously held by a dropped
    /// player), or `Err(reason)` on rejection.
    pub fn on_join_game(&mut self, request_relay_id: RelayId) -> Result<(u8, bool), String> {
        // Verify relay identity
        if request_relay_id != self.relay_id {
            return Err("Relay identity mismatch".into());
        }
        // 优先复用断线席位(重连场景),保证重连后拿回原 player_id
        if let Some(&pid) = self.disconnected.iter().min() {
            self.disconnected.remove(&pid);
            return Ok((pid, true));
        }
        // 否则按序分配新席位
        if (self.next_player_id as usize) >= self.all_players.len() {
            return Err("Room is full".into());
        }
        let player_id = self.next_player_id;
        self.next_player_id += 1;
        Ok((player_id, false))
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
        // Check if tick is already finalized (in log). log may be non-ordered if
        // ticks finalize out of order (high tick before low), so scan (not last()).
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
    /// D11: Build reconnect metadata only — the command log is delivered as
    /// `page_count` `ReconnectPage` pushes on the reliable Control channel.
    /// D2: `total_ticks` = COUNT of log entries with `tick > last_tick_consumed`
    /// (not a tick span); pages are bucketed by tick VALUE below.
    /// D12: Validates ruleset_version compatibility (done client-side on apply).
    pub fn handle_reconnect(&self, request: &ReconnectRequest) -> Result<ReconnectResponse, String> {
        // 安全:拒绝跨对局的日志请求
        if request.game_id != self.game_id {
            return Err(format!("game_id mismatch: {} != {}", request.game_id, self.game_id));
        }
        if self.frozen {
            return Err("game is frozen — reconnect rejected".into());
        }
        let total_ticks = self
            .log
            .iter()
            .filter(|b| b.tick > request.last_tick_consumed)
            .count() as u32;
        Ok(ReconnectResponse {
            game_id: self.game_id,
            ruleset_version: self.ruleset_version,
            seed: self.seed,
            map_spec_hash: self.map_spec_hash,
            map_size: self.map_size,
            first_tick: request.last_tick_consumed + 1,
            total_ticks,
            page_count: total_ticks.div_ceil(PAGE_TICKS),
            players: self.player_states(),
        })
    }

    /// Build reconnect page `page_index`: log entries whose tick falls in
    /// `[first_tick + i*PAGE_TICKS, first_tick + (i+1)*PAGE_TICKS)`.
    ///
    /// Bucketed by tick VALUE, not log position — the finalized log is
    /// append-ordered and may finalize out of tick order (D2). Returns None
    /// when `page_index >= page_count`.
    pub fn reconnect_page(&self, response: &ReconnectResponse, page_index: u32) -> Option<ReconnectPage> {
        if page_index >= response.page_count {
            return None;
        }
        let lo = response.first_tick + page_index * PAGE_TICKS;
        let hi = response.first_tick + (page_index + 1) * PAGE_TICKS;
        let ticks: Vec<TickCommands> = self
            .log
            .iter()
            .filter(|b| b.tick >= lo && b.tick < hi)
            .cloned()
            .collect();
        Some(ReconnectPage {
            page_index,
            page_count: response.page_count,
            first_tick: lo,
            ticks,
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

    pub fn map_size(&self) -> simulation::map::MapSize {
        self.map_size
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
        RelayServer::new(1, rid, 1, 42, 0xABC, simulation::map::MapSize::Medium, vec![0, 1], 3, 1000)
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
        assert_eq!(resp.total_ticks, 1); // Only tick 2
        assert_eq!(resp.page_count, 1);
        assert_eq!(resp.seed, 42);
        let page = relay.reconnect_page(&resp, 0).unwrap();
        assert_eq!(page.page_index, 0);
        assert_eq!(page.page_count, 1);
        assert_eq!(page.ticks.len(), 1);
        assert_eq!(page.ticks[0].tick, 2);
        assert!(relay.reconnect_page(&resp, 1).is_none()); // out of range
    }

    #[test]
    fn test_reconnect_pagination_boundaries() {
        let mut relay = relay_2p();
        let now = 1000;
        // Finalize ticks 1..=65 in order
        for tick in 1u32..=65 {
            let r = relay.on_player_frame(&make_empty_frame(tick, 0, tick as u64), now + tick as u64);
            assert!(r.0.is_none(), "tick {} waits for player 1", tick);
            let r = relay.on_player_frame(&make_empty_frame(tick, 1, tick as u64 + 100), now + tick as u64);
            assert!(r.0.is_some(), "tick {} should finalize", tick);
        }
        assert_eq!(relay.command_log().len(), 65);

        // Reconnect from tick 0 → first=1, 65 ticks, 3 pages (32/32/1)
        let resp = relay
            .handle_reconnect(&ReconnectRequest { game_id: 1, last_tick_consumed: 0 })
            .unwrap();
        assert_eq!(resp.first_tick, 1);
        assert_eq!(resp.total_ticks, 65);
        assert_eq!(resp.page_count, 3);

        let p0 = relay.reconnect_page(&resp, 0).unwrap();
        assert_eq!(p0.ticks.len(), 32);
        assert_eq!(p0.first_tick, 1);
        assert_eq!(p0.ticks[0].tick, 1);
        assert_eq!(p0.ticks[31].tick, 32);
        let p1 = relay.reconnect_page(&resp, 1).unwrap();
        assert_eq!(p1.ticks.len(), 32);
        assert_eq!(p1.first_tick, 33);
        assert_eq!(p1.ticks[0].tick, 33);
        assert_eq!(p1.ticks[31].tick, 64);
        let p2 = relay.reconnect_page(&resp, 2).unwrap();
        assert_eq!(p2.ticks.len(), 1);
        assert_eq!(p2.first_tick, 65);
        assert_eq!(p2.ticks[0].tick, 65);
        assert!(relay.reconnect_page(&resp, 3).is_none()); // out of range
    }

    #[test]
    fn test_reconnect_buckets_out_of_order_log() {
        let mut relay = relay_2p();
        let now = 1000;
        // 乱序定稿:5 → 3 → 4(log append 序 [5,3,4],非 tick 序)
        relay.on_player_frame(&make_empty_frame(5, 0, 1), now);
        let r5 = relay.on_player_frame(&make_empty_frame(5, 1, 2), now);
        assert!(r5.0.is_some());
        relay.on_player_frame(&make_empty_frame(3, 0, 3), now);
        let r3 = relay.on_player_frame(&make_empty_frame(3, 1, 4), now);
        assert!(r3.0.is_some());
        relay.on_player_frame(&make_empty_frame(4, 0, 5), now);
        let r4 = relay.on_player_frame(&make_empty_frame(4, 1, 6), now);
        assert!(r4.0.is_some());
        let log_ticks: Vec<u32> = relay.command_log().iter().map(|b| b.tick).collect();
        assert_eq!(log_ticks, vec![5, 3, 4]);

        // Reconnect from tick 2 → total=3,单页按值覆盖 tick 3,4,5
        let resp = relay
            .handle_reconnect(&ReconnectRequest { game_id: 1, last_tick_consumed: 2 })
            .unwrap();
        assert_eq!(resp.first_tick, 3);
        assert_eq!(resp.total_ticks, 3);
        assert_eq!(resp.page_count, 1);
        let page = relay.reconnect_page(&resp, 0).unwrap();
        let mut page_ticks: Vec<u32> = page.ticks.iter().map(|b| b.tick).collect();
        // 按值归桶:包含 3,4,5 各一次(页内顺序无关,客户端按 tick 键入 HashMap)
        page_ticks.sort_unstable();
        assert_eq!(page_ticks, vec![3, 4, 5]);
    }

    #[test]
    fn test_reconnect_edges_empty_and_frozen() {
        let mut relay = relay_2p();
        let now = 1000;
        // 无定稿 → total_ticks=0, page_count=0,无页面
        let resp0 = relay
            .handle_reconnect(&ReconnectRequest { game_id: 1, last_tick_consumed: 0 })
            .unwrap();
        assert_eq!(resp0.total_ticks, 0);
        assert_eq!(resp0.page_count, 0);
        assert!(relay.reconnect_page(&resp0, 0).is_none());

        // 全体掉线 → frozen → reconnect 拒绝
        relay.on_disconnect(0);
        relay.on_disconnect(1);
        relay.on_full_disconnect(now);
        let resp = relay.handle_reconnect(&ReconnectRequest { game_id: 1, last_tick_consumed: 0 });
        assert!(resp.is_err(), "frozen game must reject reconnect");
    }

    #[test]
    fn test_reconnect_resume_no_overlap() {
        let mut relay = relay_2p();
        let now = 1000;
        // Finalize ticks 1..=100
        for tick in 1u32..=100 {
            relay.on_player_frame(&make_empty_frame(tick, 0, tick as u64), now + tick as u64);
            let r = relay.on_player_frame(&make_empty_frame(tick, 1, tick as u64 + 100), now + tick as u64);
            assert!(r.0.is_some());
        }

        // 第一次重连:从 tick 10 起,74 ticks → 3 页(32/32/10)
        let resp1 = relay
            .handle_reconnect(&ReconnectRequest { game_id: 1, last_tick_consumed: 10 })
            .unwrap();
        assert_eq!(resp1.first_tick, 11);
        assert_eq!(resp1.total_ticks, 90);
        assert_eq!(resp1.page_count, 3);
        // 应用前 2 页(至 tick 74)→ 模拟已推进
        let applied: Vec<u32> = relay
            .reconnect_page(&resp1, 0)
            .unwrap()
            .ticks
            .iter()
            .chain(relay.reconnect_page(&resp1, 1).unwrap().ticks.iter())
            .map(|b| b.tick)
            .collect();
        let last_applied = *applied.iter().max().unwrap();
        assert_eq!(last_applied, 74);

        // 再掉线,从 last_applied 续传:first_tick = 75,无重叠无缺口
        let resp2 = relay
            .handle_reconnect(&ReconnectRequest { game_id: 1, last_tick_consumed: last_applied })
            .unwrap();
        assert_eq!(resp2.first_tick, 75);
        assert_eq!(resp2.total_ticks, 26); // 75..=100
        assert_eq!(resp2.page_count, 1);
        let page = relay.reconnect_page(&resp2, 0).unwrap();
        assert_eq!(page.ticks.len(), 26);
        assert_eq!(page.ticks[0].tick, 75);
        assert_eq!(page.ticks[25].tick, 100);
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
        assert_eq!(relay.on_join_game(crate::discovery::RelayId(42)).unwrap(), (0, false));
        assert_eq!(relay.on_join_game(crate::discovery::RelayId(42)).unwrap(), (1, false));
        // 玩家 0 掉线:席位保留,状态标 Disconnected
        let states = relay.on_disconnect(0);
        assert!(states.iter().any(|s| matches!(s, PlayerState::Disconnected { player_id: 0 })));
        // 重连:复用原 player_id 0(而不是 Room is full),标记为重连
        assert_eq!(relay.on_join_game(crate::discovery::RelayId(42)).unwrap(), (0, true));
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

    #[test]
    fn test_out_of_order_tick_finalize_no_duplicate() {
        // UDP 乱序:高 tick 先到先定稿,低 tick 后到——try_finalize 不得重复定稿
        let mut relay = relay_2p();
        let now = 1000;
        // 先提交 tick 5(乱序,高 tick 先到)
        relay.on_player_frame(&make_empty_frame(5, 0, 1), now);
        let r5 = relay.on_player_frame(&make_empty_frame(5, 1, 2), now);
        assert!(r5.0.is_some(), "tick 5 finalizes first (out of order)");
        // 后提交 tick 3(迟到)
        relay.on_player_frame(&make_empty_frame(3, 0, 3), now);
        let r3 = relay.on_player_frame(&make_empty_frame(3, 1, 4), now);
        assert!(r3.0.is_some(), "tick 3 finalizes later");
        // log 含 3 和 5,各一次(不重复)
        let ticks: Vec<u32> = relay.command_log().iter().map(|b| b.tick).collect();
        assert!(ticks.contains(&3) && ticks.contains(&5));
        assert_eq!(ticks.iter().filter(|t| **t == 3).count(), 1);
        assert_eq!(ticks.iter().filter(|t| **t == 5).count(), 1);
        // 再次提交 tick 5(重复迟到帧)→ 不重复定稿
        let late = relay.on_player_frame(&make_empty_frame(5, 0, 9), now);
        assert!(late.0.is_none(), "already-finalized tick 5 not re-finalized");
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
    fn test_apply_reconnect_metadata_then_pages() {
        let mut source = NetworkCommandSource::default();
        // 版本不匹配 → Err(D12)
        let resp_bad = ReconnectResponse {
            game_id: 1,
            ruleset_version: 2,
            seed: 1,
            map_spec_hash: 0,
            map_size: simulation::map::MapSize::Medium,
            first_tick: 1,
            total_ticks: 0,
            page_count: 0,
            players: vec![],
        };
        assert!(source.apply_reconnect(&resp_bad, 1).is_err());

        // 版本匹配,33 ticks → 2 页(32/1)
        let first = 2u32;
        let total = PAGE_TICKS + 1;
        let resp = ReconnectResponse {
            game_id: 1,
            ruleset_version: 1,
            seed: 42,
            map_spec_hash: 0,
            map_size: simulation::map::MapSize::Medium,
            first_tick: first,
            total_ticks: total,
            page_count: total.div_ceil(PAGE_TICKS),
            players: vec![],
        };
        source.apply_reconnect(&resp, 1).unwrap();
        assert!(source.reconnect_meta.is_some());
        assert_eq!(source.game_id, 1);
        assert!(source.connected);
        // 页面未应用前 tick 不可用
        assert!(!source.is_tick_ready(first));

        // 逐页应用(页内 tick 由 [lo, hi) 构成,与 D2 分桶一致)
        for i in 0..resp.page_count {
            let lo = first + i * PAGE_TICKS;
            let hi = first + (i + 1) * PAGE_TICKS;
            let ticks: Vec<TickCommands> = (lo..hi).map(|t| make_tick(t, 0)).collect();
            let page = ReconnectPage {
                page_index: i,
                page_count: resp.page_count,
                first_tick: lo,
                ticks,
            };
            source.apply_reconnect_page(&page).unwrap();
        }
        // 末页后游标清除
        assert!(source.reconnect_meta.is_none());
        // 全部 tick 就绪
        for t in first..(first + total) {
            assert!(source.is_tick_ready(t));
        }
    }

    #[test]
    fn test_apply_reconnect_page_validation_rejects_stale() {
        let mut source = NetworkCommandSource::default();
        let meta = ReconnectResponse {
            game_id: 1,
            ruleset_version: 1,
            seed: 42,
            map_spec_hash: 0,
            map_size: simulation::map::MapSize::Medium,
            first_tick: 100,
            total_ticks: PAGE_TICKS,
            page_count: 1,
            players: vec![],
        };
        source.apply_reconnect(&meta, 1).unwrap();

        // 无元数据 → 拒绝
        let mut orphan = NetworkCommandSource::default();
        assert!(orphan.apply_reconnect_page(&ReconnectPage {
            page_index: 0,
            page_count: 1,
            first_tick: 100,
            ticks: vec![],
        }).is_err());

        // page_count 与元数据不符 → 拒绝(旧会话 stale 页)
        assert!(source.apply_reconnect_page(&ReconnectPage {
            page_index: 0,
            page_count: 2,
            first_tick: 100,
            ticks: vec![],
        }).is_err());

        // page_index 越界 → 拒绝
        assert!(source.apply_reconnect_page(&ReconnectPage {
            page_index: 1,
            page_count: 1,
            first_tick: 100,
            ticks: vec![],
        }).is_err());

        // 正常页 → 接受
        let ok = ReconnectPage {
            page_index: 0,
            page_count: 1,
            first_tick: 100,
            ticks: (100..100 + PAGE_TICKS).map(|t| make_tick(t, 0)).collect(),
        };
        source.apply_reconnect_page(&ok).unwrap();
        assert!(source.reconnect_meta.is_none());

        // 末页后再来重复页 → 无元数据,拒绝
        assert!(source.apply_reconnect_page(&ok).is_err());
    }

    #[test]
    fn test_apply_reconnect_limited_clear_preserves_live_broadcast() {
        let mut source = NetworkCommandSource::default();
        // 重连前:陈旧 tick(1)+ 实时 broadcast tick(150,页面范围外)
        source.relay_buffer.insert(1, make_tick(1, 0));
        source.relay_buffer.insert(150, make_tick(150, 0));
        let meta = ReconnectResponse {
            game_id: 1,
            ruleset_version: 1,
            seed: 42,
            map_spec_hash: 0,
            map_size: simulation::map::MapSize::Medium,
            first_tick: 100,
            total_ticks: PAGE_TICKS,
            page_count: 1,
            players: vec![],
        };
        source.apply_reconnect(&meta, 1).unwrap();
        // 陈旧项被清,实时项保留(D3 限定 clear)
        assert!(!source.is_tick_ready(1));
        assert!(source.is_tick_ready(150));
        assert!(source.reconnect_meta.is_some());
    }

    #[test]
    fn test_apply_reconnect_zero_ticks_completes_immediately() {
        let mut source = NetworkCommandSource::default();
        let meta = ReconnectResponse {
            game_id: 1,
            ruleset_version: 1,
            seed: 42,
            map_spec_hash: 0,
            map_size: simulation::map::MapSize::Medium,
            first_tick: 50,
            total_ticks: 0,
            page_count: 0,
            players: vec![],
        };
        source.apply_reconnect(&meta, 1).unwrap();
        assert!(source.reconnect_meta.is_none()); // 无页可拉,立即完成
    }
}

// ═══════════════════════════════════════════════════════════════
// LanDiscovery — see `crate::discovery` for the new model-based impl
// ═══════════════════════════════════════════════════════════════

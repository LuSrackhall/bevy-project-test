//! Shared relay runtime — UDP socket + per-client reliable sessions + tick broadcast.
//!
//! Used by both `ThreadRelayRuntime` (embedded in the game process)
//! and the standalone `relay` CLI binary.
//!
//! Architecture: a single dual-stack UDP socket receives datagrams for all
//! clients. A shared recv loop demultiplexes by source address into per-client
//! `ReliableSocket` sessions (each on its own task). Message handling mirrors
//! the previous TCP flow; only the transport is reliable-UDP. Sessions are
//! shared via `tokio::sync::Mutex` so the reliable socket can be polled across
//! awaits (its guard is Send).

use std::collections::{HashMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::sync::Mutex as AsyncMutex;

use crate::discovery::RelayId;
use crate::network::{
    BroadcastFrame, LobbyPlayerState, RelayClientMessage, RelayServer, RelayServerMessage,
};
use crate::reliable_udp::channel::DatagramChannel;
use crate::reliable_udp::{ReliableConfig, ReliableSocket};

/// Static relay configuration passed at start.
#[derive(Clone, Debug)]
pub struct RelayConfig {
    pub relay_id: RelayId,
    pub game_id: u64,
    pub ruleset_version: u32,
    pub seed: u64,
    pub map_spec_hash: u64,
    /// Map size (Scene B rebuild needs it in reconnect metadata).
    pub map_size: simulation::map::MapSize,
    pub player_count: u8,
    pub input_delay: u32,
    /// Live connected-client count, shared with the LAN beacon for current_players.
    pub current_clients: Arc<AtomicUsize>,
}

/// `DatagramChannel` for one relay client: sends over the shared socket,
/// receives from the session's own inbound queue (filled by the recv loop).
struct RelayChannel {
    shared: Arc<UdpSocket>,
    inbound: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

#[async_trait]
impl DatagramChannel for RelayChannel {
    async fn send_to(&mut self, buf: &[u8], to: SocketAddr) -> io::Result<()> {
        self.shared.send_to(buf, to).await.map(|_| ())
    }

    async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let data = self.inbound.lock().unwrap().pop_front();
        match data {
            Some(d) => {
                let n = d.len().min(buf.len());
                buf[..n].copy_from_slice(&d[..n]);
                Ok((n, SocketAddr::from(([0, 0, 0, 0], 0))))
            }
            None => Err(io::Error::new(io::ErrorKind::WouldBlock, "session inbound empty")),
        }
    }
}

/// Per-client session (player identity + reliable socket + inbound queue).
struct RelaySession {
    socket: ReliableSocket,
    inbound: Arc<Mutex<VecDeque<Vec<u8>>>>,
    player_id: Option<u8>,
    last_seen_ms: Arc<AtomicU64>,
}

/// Shared relay context, accessed by all session tasks.
struct RelayCtx {
    server: Mutex<RelayServer>,
    clients: Mutex<HashMap<u8, Arc<AsyncMutex<RelaySession>>>>,
    player_count: u8,
    current_clients: Arc<AtomicUsize>,
}

impl RelayCtx {
    fn new(server: RelayServer, player_count: u8, current_clients: Arc<AtomicUsize>) -> Arc<Self> {
        Arc::new(Self {
            server: Mutex::new(server),
            clients: Mutex::new(HashMap::new()),
            player_count,
            current_clients,
        })
    }
}

fn now_ms_ts() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Encode a relay→client message and stage it on every connected session's
/// reliable socket (Control channel, CH=1). Flushed by each session's poll().
async fn broadcast(ctx: &Arc<RelayCtx>, msg: &RelayServerMessage) {
    let Ok(data) = bincode::serde::encode_to_vec(msg, bincode::config::standard()) else {
        return;
    };
    // Collect session refs under the short guard, then await each lock outside it.
    let sessions: Vec<Arc<AsyncMutex<RelaySession>>> = {
        let clients = ctx.clients.lock().unwrap();
        clients.values().cloned().collect()
    };
    for session in sessions {
        session.lock().await.socket.send_reliable(1, data.clone());
    }
}

/// Run the relay on an already-bound UDP socket (dual-stack).
pub async fn run_relay(socket: UdpSocket, config: RelayConfig, stop: &AtomicBool) {
    let now_ms = now_ms_ts();

    let server = RelayServer::new(
        config.game_id,
        config.relay_id,
        config.ruleset_version,
        config.seed,
        config.map_spec_hash,
        config.map_size,
        (0..config.player_count).collect(),
        config.input_delay,
        now_ms,
    );

    let ctx = RelayCtx::new(server, config.player_count, config.current_clients.clone());
    let shared = Arc::new(socket);
    eprintln!(
        "[RELAY] Relay ready (players={}, seed={})",
        config.player_count, config.seed
    );

    let hb_timeout_ms = 1500u64; // 3 × 500ms heartbeat

    let mut sessions: HashMap<SocketAddr, Arc<AsyncMutex<RelaySession>>> = HashMap::new();

    let mut buf = [0u8; 65535];
    loop {
        tokio::select! {
            r = shared.recv_from(&mut buf) => {
                match r {
                    Ok((n, from)) => {
                        let data = buf[..n].to_vec();
                        if let Some(session) = sessions.get(&from) {
                            let mut s = session.lock().await;
                            let mut inbound = s.inbound.lock().unwrap();
                            if inbound.len() < 1024 {
                                inbound.push_back(data);
                            }
                            drop(inbound);
                            s.last_seen_ms.store(now_ms_ts(), Ordering::Relaxed);
                        } else {
                            // Unknown source: start a session; identity assigned on JoinGame.
                            let inbound = Arc::new(Mutex::new(VecDeque::new()));
                            inbound.lock().unwrap().push_back(data);
                            let last_seen = Arc::new(AtomicU64::new(now_ms_ts()));
                            let channel = Box::new(RelayChannel { shared: shared.clone(), inbound: inbound.clone() });
                            let socket = ReliableSocket::new(channel, from, ReliableConfig::default());
                            let session = Arc::new(AsyncMutex::new(RelaySession {
                                socket,
                                inbound: inbound.clone(),
                                player_id: None,
                                last_seen_ms: last_seen.clone(),
                            }));
                            sessions.insert(from, session.clone());
                            let ctx2 = ctx.clone();
                            tokio::spawn(session_task(ctx2, session.clone()));
                        }
                    }
                    Err(_) => {}
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Heartbeat sweep: drop sessions that missed their heartbeats.
                let now = now_ms_ts();
                let mut dead: Vec<SocketAddr> = Vec::new();
                for (addr, session) in &sessions {
                    let last = session.lock().await.last_seen_ms.load(Ordering::Relaxed);
                    if now.saturating_sub(last) > hb_timeout_ms {
                        dead.push(*addr);
                    }
                }
                for addr in dead {
                    if let Some(session) = sessions.remove(&addr) {
                        let pid = session.lock().await.player_id;
                        if let Some(pid) = pid {
                            // Only disconnect if this is still the current session for pid
                            // (a reconnected client may have replaced it).
                            let is_current = ctx.clients.lock().unwrap().get(&pid).map_or(false, |c| Arc::ptr_eq(c, &session));
                            if is_current {
                                ctx.current_clients.fetch_sub(1, Ordering::Relaxed);
                                let mut server = ctx.server.lock().unwrap();
                                server.on_disconnect(pid);
                                ctx.clients.lock().unwrap().remove(&pid);
                                eprintln!("[RELAY] heartbeat timeout: player {} disconnected", pid);
                            }
                        }
                    }
                }
                if stop.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
    }

    eprintln!("[RELAY] Shutting down");
}

/// One session's loop: poll the reliable socket, handle messages, keep alive.
async fn session_task(ctx: Arc<RelayCtx>, session: Arc<AsyncMutex<RelaySession>>) {
    let start = std::time::Instant::now();
    loop {
        let msgs = {
            let mut s = session.lock().await;
            s.socket.set_now(start.elapsed()); // drive RTO so relay→client retransmits
            s.socket.process(); // move due frames (incl. messages staged by handle_message) to outbound
            if s.socket.poll().await.is_err() {
                break;
            }
            s.socket.take_messages()
        };
        for bytes in msgs {
            handle_message(&ctx, &session, &bytes).await;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Handle a single decoded client message for a session.
async fn handle_message(ctx: &Arc<RelayCtx>, session: &Arc<AsyncMutex<RelaySession>>, bytes: &[u8]) {
    let Ok((request, _)) = bincode::serde::decode_from_slice::<RelayClientMessage, _>(
        bytes, bincode::config::standard(),
    ) else {
        return;
    };
    match request {
        RelayClientMessage::JoinGame { room_id: _, relay_id } => {
            let result = { ctx.server.lock().unwrap().on_join_game(relay_id) };
            match result {
                Ok((pid, reused)) => {
                    let mut s = session.lock().await;
                    s.player_id = Some(pid);
                    ctx.clients.lock().unwrap().insert(pid, session.clone());
                    ctx.current_clients.fetch_add(1, Ordering::Relaxed);
                    let msg = RelayServerMessage::GameJoined {
                        game_id: 1,
                        player_id: pid,
                        player_count: ctx.player_count,
                    };
                    if let Ok(data) = bincode::serde::encode_to_vec(&msg, bincode::config::standard()) {
                        s.socket.send_reliable(1, data);
                    }
                    // Scene B: a player reusing a dropped seat in a started game is a
                    // restarted process — re-send GameStarted so its lobby transitions
                    // to Playing and rebuilds from the reconnect metadata seed.
                    if reused {
                        let (started, game_started, map_size) = {
                            let server = ctx.server.lock().unwrap();
                            (server.is_game_started(), server.seed(), server.map_size())
                        };
                        if started {
                            let started = RelayServerMessage::GameStarted {
                                game_id: 1,
                                seed: game_started,
                                map_size,
                                player_count: ctx.player_count,
                            };
                            if let Ok(data) = bincode::serde::encode_to_vec(&started, bincode::config::standard()) {
                                s.socket.send_reliable(1, data);
                            }
                            eprintln!("[RELAY] re-sent GameStarted to reconnecting player {}", pid);
                        }
                    }
                }
                Err(reason) => {
                    let reject = RelayServerMessage::JoinRejected { reason };
                    if let Ok(data) = bincode::serde::encode_to_vec(&reject, bincode::config::standard()) {
                        session.lock().await.socket.send_reliable(1, data);
                    }
                }
            }
        }
        RelayClientMessage::PlayerTick(frame) => {
            let now_ms = now_ms_ts();
            let (batch, game_just_started) = {
                let mut server = ctx.server.lock().unwrap();
                server.on_player_frame(&frame, now_ms)
            };

            if game_just_started {
                let (seed, map_size) = {
                    let server = ctx.server.lock().unwrap();
                    (server.seed(), server.map_size())
                };
                let started = RelayServerMessage::GameStarted {
                    game_id: 1,
                    seed,
                    map_size,
                    player_count: ctx.player_count,
                };
                eprintln!("[RELAY] Broadcasting GameStarted (seed={}, players={})", seed, ctx.player_count);
                broadcast(ctx, &started).await;
            }

            if let Some(tick_cmds) = batch {
                let bc = RelayServerMessage::Broadcast(BroadcastFrame {
                    game_id: 1,
                    ruleset_version: 1,
                    payload: tick_cmds,
                    relay_ts_ms: now_ms,
                });
                broadcast(ctx, &bc).await;
            }
        }
        RelayClientMessage::Reconnect(req) => {
            let resp = { ctx.server.lock().unwrap().handle_reconnect(&req) };
            match resp {
                Ok(meta) => {
                    // Metadata first, then page_count pages on the reliable
                    // Control channel (ReliableOrdered preserves page order).
                    if let Ok(data) = bincode::serde::encode_to_vec(
                        &RelayServerMessage::ReconnectResponse(meta.clone()),
                        bincode::config::standard(),
                    ) {
                        session.lock().await.socket.send_reliable(1, data);
                    }
                    for page_index in 0..meta.page_count {
                        let page = { ctx.server.lock().unwrap().reconnect_page(&meta, page_index) };
                        if let Some(page) = page {
                            if let Ok(data) = bincode::serde::encode_to_vec(
                                &RelayServerMessage::ReconnectPage(page),
                                bincode::config::standard(),
                            ) {
                                session.lock().await.socket.send_reliable(1, data);
                            }
                        }
                    }
                }
                Err(e) => {
                    let msg = RelayServerMessage::Error { code: 1, message: e };
                    if let Ok(data) = bincode::serde::encode_to_vec(&msg, bincode::config::standard()) {
                        session.lock().await.socket.send_reliable(1, data);
                    }
                }
            }
        }
        RelayClientMessage::LobbyReady { game_id, player_id, ready, map_size: _ } => {
            let all_ready = {
                let mut server = ctx.server.lock().unwrap();
                if server.is_game_started() {
                    false
                } else if ready {
                    server.on_lobby_ready(player_id)
                } else {
                    server.on_lobby_not_ready(player_id);
                    false
                }
            };

            let lobby_players = {
                let server = ctx.server.lock().unwrap();
                let players: Vec<LobbyPlayerState> = ctx.clients.lock().unwrap().keys().map(|pid| {
                    LobbyPlayerState {
                        player_id: *pid,
                        ready: server.is_player_ready(*pid),
                        selected_map: None,
                    }
                }).collect();
                players
            };
            let update = RelayServerMessage::LobbyUpdate { game_id, players: lobby_players };
            broadcast(ctx, &update).await;

            if all_ready {
                let (seed, map_size) = {
                    let server = ctx.server.lock().unwrap();
                    (server.seed(), server.map_size())
                };
                let started = RelayServerMessage::GameStarted {
                    game_id,
                    seed,
                    map_size,
                    player_count: ctx.player_count,
                };
                eprintln!("[RELAY] All players ready! Starting game (seed={})", seed);
                broadcast(ctx, &started).await;
            }
        }
    }
}

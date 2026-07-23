//! Shared relay runtime — TCP accept + client handler + tick broadcast.
//!
//! Used by both `ThreadRelayRuntime` (embedded in the game process)
//! and the standalone `relay` CLI binary.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::discovery::RelayId;
use crate::network::{
    BroadcastFrame, LobbyPlayerState, RelayClientMessage, RelayServer, RelayServerMessage,
};

/// Static relay configuration passed at start.
#[derive(Clone, Debug)]
pub struct RelayConfig {
    pub relay_id: RelayId,
    pub game_id: u64,
    pub ruleset_version: u32,
    pub seed: u64,
    pub map_spec_hash: u64,
    pub player_count: u8,
    pub input_delay: u32,
}

/// Shared relay context, accessed by all connection tasks.
struct RelayCtx {
    server: Mutex<RelayServer>,
    clients: Mutex<HashMap<u8, mpsc::UnboundedSender<RelayServerMessage>>>,
    player_count: u8,
}

impl RelayCtx {
    fn new(server: RelayServer, player_count: u8) -> Arc<Self> {
        Arc::new(Self {
            server: Mutex::new(server),
            clients: Mutex::new(HashMap::new()),
            player_count,
        })
    }
}

/// Run the full relay accept + handle loop on an already-bound TcpListener.
///
/// Accepts incoming connections, handles JoinGame/PlayerTick/LobbyReady,
/// and broadcasts finalized ticks. Exits when `stop` is set to true.
pub async fn run_relay(
    listener: TcpListener,
    config: RelayConfig,
    stop: &AtomicBool,
) {
    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let server = RelayServer::new(
        config.game_id,
        config.relay_id,
        config.ruleset_version,
        config.seed,
        config.map_spec_hash,
        (0..config.player_count).collect(),
        config.input_delay,
        now_ms,
    );

    let ctx = RelayCtx::new(server, config.player_count);
    eprintln!(
        "[RELAY] Relay ready (players={}, seed={})",
        config.player_count, config.seed
    );

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        eprintln!("[RELAY] Connect from {}", addr);
                        let ctx = Arc::clone(&ctx);
                        tokio::spawn(handle_client(ctx, stream));
                    }
                    Err(e) => {
                        eprintln!("[RELAY] Accept error: {}", e);
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
    }

    eprintln!("[RELAY] Shutting down");
}

/// Handle a single client connection.
async fn handle_client(ctx: Arc<RelayCtx>, stream: TcpStream) {
    let (mut reader, writer) = tokio::io::split(stream);

    // Step 1: Wait for JoinGame message before assigning identity
    let player_id = {
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).await.is_err() {
            return;
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if reader.read_exact(&mut buf).await.is_err() {
            return;
        }
        let Ok((request, _)) = bincode::serde::decode_from_slice::<RelayClientMessage, _>(
            &buf, bincode::config::standard(),
        ) else {
            return;
        };

        match request {
            RelayClientMessage::JoinGame { room_id: _, relay_id } => {
                let result = {
                    let mut server = ctx.server.lock().unwrap();
                    server.on_join_game(relay_id)
                };
                match result {
                    Ok(pid) => pid,
                    Err(reason) => {
                        let reject = RelayServerMessage::JoinRejected { reason };
                        let mut w = writer;
                        relay_write(&mut w, &reject).await;
                        return;
                    }
                }
            }
            _ => {
                eprintln!("[RELAY] Expected JoinGame, got unexpected message");
                return;
            }
        }
    };

    // Step 2: Register client
    let (tx, mut rx) = mpsc::unbounded_channel::<RelayServerMessage>();
    {
        let mut clients = ctx.clients.lock().unwrap();
        clients.insert(player_id, tx);
    }

    // Step 3: Send GameJoined with assigned player_id
    let msg = RelayServerMessage::GameJoined {
        game_id: 1,
        player_id,
        player_count: ctx.player_count,
    };
    let mut w = writer;
    relay_write(&mut w, &msg).await;

    // Writer task: forward messages from channels to TCP
    let write_h = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            relay_write(&mut w, &msg).await;
        }
    });

    // Reader: parse frames, dispatch
    let mut len_buf = [0u8; 4];
    loop {
        if reader.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if reader.read_exact(&mut buf).await.is_err() {
            break;
        }
        let Ok((request, _)) = bincode::serde::decode_from_slice::<RelayClientMessage, _>(
            &buf, bincode::config::standard(),
        ) else {
            continue;
        };

        match request {
            RelayClientMessage::JoinGame { .. } => {
                // Already joined — ignore redundant JoinGame
            }
            RelayClientMessage::PlayerTick(frame) => {
                let now_ms = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let (batch, game_just_started) = {
                    let mut server = ctx.server.lock().unwrap();
                    server.on_player_frame(&frame, now_ms)
                };

                // Broadcast GameStarted when all players have connected
                if game_just_started {
                    let seed = {
                        let server = ctx.server.lock().unwrap();
                        server.seed()
                    };
                    let started_msg = RelayServerMessage::GameStarted {
                        game_id: 1,
                        seed,
                        player_count: ctx.player_count,
                    };
                    eprintln!("[RELAY] Broadcasting GameStarted (seed={}, players={})", seed, ctx.player_count);
                    let clients = ctx.clients.lock().unwrap();
                    for sender in clients.values() {
                        let _ = sender.send(started_msg.clone());
                    }
                }

                if let Some(tick_cmds) = batch {
                    let broadcast = RelayServerMessage::Broadcast(BroadcastFrame {
                        game_id: 1,
                        ruleset_version: 1,
                        payload: tick_cmds,
                        relay_ts_ms: now_ms,
                    });
                    let clients = ctx.clients.lock().unwrap();
                    for sender in clients.values() {
                        let _ = sender.send(broadcast.clone());
                    }
                }
            }
            RelayClientMessage::Reconnect(req) => {
                let resp = {
                    let server = ctx.server.lock().unwrap();
                    server.handle_reconnect(&req)
                };
                let clients = ctx.clients.lock().unwrap();
                if let Some(sender) = clients.get(&player_id) {
                    match resp {
                        Ok(r) => { let _ = sender.send(RelayServerMessage::ReconnectResponse(r)); }
                        Err(e) => { let _ = sender.send(RelayServerMessage::Error { code: 1, message: e }); }
                    }
                }
            }
            RelayClientMessage::LobbyReady { game_id, player_id, ready, map_size: _ } => {
                if !ready { continue; }
                let all_ready = {
                    let mut server = ctx.server.lock().unwrap();
                    server.on_lobby_ready(player_id)
                };

                // Broadcast LobbyUpdate to all connected clients
                {
                    let server = ctx.server.lock().unwrap();
                    let lobby_players: Vec<LobbyPlayerState> = ctx.clients.lock().unwrap().keys().map(|pid| {
                        let mask = server.lobby_ready_mask();
                        LobbyPlayerState {
                            player_id: *pid,
                            ready: (mask >> pid) & 1 == 1,
                            selected_map: None,
                        }
                    }).collect();
                    let update = RelayServerMessage::LobbyUpdate { game_id, players: lobby_players };
                    let clients = ctx.clients.lock().unwrap();
                    for sender in clients.values() {
                        let _ = sender.send(update.clone());
                    }
                }

                // If all players are ready, start the game
                if all_ready {
                    let seed = {
                        let server = ctx.server.lock().unwrap();
                        server.seed()
                    };
                    let started_msg = RelayServerMessage::GameStarted {
                        game_id,
                        seed,
                        player_count: ctx.player_count,
                    };
                    eprintln!("[RELAY] All players ready! Starting game (seed={})", seed);
                    let clients = ctx.clients.lock().unwrap();
                    for sender in clients.values() {
                        let _ = sender.send(started_msg.clone());
                    }
                }
            }
        }
    }

    eprintln!("Player {} disconnected", player_id);
    write_h.abort();
    {
        let mut server = ctx.server.lock().unwrap();
        server.on_disconnect(player_id);
    }
}

/// Write a length-prefixed bincode frame to a TCP stream.
async fn relay_write(writer: &mut (impl tokio::io::AsyncWrite + Unpin), msg: &RelayServerMessage) {
    if let Ok(data) = bincode::serde::encode_to_vec(msg, bincode::config::standard()) {
        let len_bytes = (data.len() as u32).to_le_bytes();
        let _ = writer.write_all(&len_bytes).await;
        let _ = writer.write_all(&data).await;
    }
}

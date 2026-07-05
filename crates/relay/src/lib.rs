//! Relay library — starts a TCP relay server for RTS CommandStream Protocol v1.0.
//!
//! Provides `start_relay()` that accepts a port, seed, and player count,
//! and returns a future that runs the relay until stopped.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use bevy_adapter::network::{
    BroadcastFrame, RelayClientMessage, RelayServerMessage, RelayServer,
};

/// Shared relay context, accessed by all connection tasks.
struct RelayCtx {
    server: Mutex<RelayServer>,
    clients: Mutex<HashMap<u8, mpsc::UnboundedSender<RelayServerMessage>>>,
    player_count: u8,
    next_player_id: Mutex<u8>,
}

impl RelayCtx {
    fn new(server: RelayServer, player_count: u8) -> Arc<Self> {
        Arc::new(Self {
            server: Mutex::new(server),
            clients: Mutex::new(HashMap::new()),
            player_count,
            next_player_id: Mutex::new(0),
        })
    }
}

/// Start the relay server. Accepts connections until `shutdown` is set to true.
pub async fn start_relay(
    port: u16,
    seed: u64,
    player_count: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_millis() as u64;

    let server = RelayServer::new(
        1, 1, seed, 0, (0..player_count).collect(), 3, now_ms,
    );

    let ctx = RelayCtx::new(server, player_count);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("Relay on port {} (players={}, seed={}) [v2: game_started]", port, player_count, seed);

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Connect from {}", addr);
        let ctx = Arc::clone(&ctx);
        tokio::spawn(handle(ctx, stream));
    }
}

/// Handle a single client connection.
async fn handle(ctx: Arc<RelayCtx>, stream: tokio::net::TcpStream) {
    let player_id = {
        let mut next = ctx.next_player_id.lock().unwrap();
        if *next >= ctx.player_count {
            eprintln!("Game full");
            return;
        }
        let pid = *next;
        *next += 1;
        pid
    };

    let (tx, mut rx) = mpsc::unbounded_channel::<RelayServerMessage>();
    {
        let mut clients = ctx.clients.lock().unwrap();
        clients.insert(player_id, tx);
    }

    let (mut reader, writer) = tokio::io::split(stream);

    // Send GameJoined
    let msg = RelayServerMessage::GameJoined { game_id: 1, player_id };
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
            RelayClientMessage::JoinGame(_) => {}
        }
    }

    println!("Player {} disconnected", player_id);
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

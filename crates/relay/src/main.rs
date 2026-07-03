//! Relay server — standalone TCP binary for RTS CommandStream Protocol v1.0.
//!
//! Usage: relay --port <port> --seed <seed> --players <count>

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use bevy_adapter::network::{
    BroadcastFrame, PlayerTickFrame, RelayClientMessage, RelayServerMessage, RelayServer,
};

struct RelayContext {
    server: Mutex<RelayServer>,
    clients: Mutex<HashMap<u8, mpsc::UnboundedSender<RelayServerMessage>>>,
    player_count: u8,
    next_player_id: Mutex<u8>,
}

impl RelayContext {
    fn new(server: RelayServer, player_count: u8) -> Arc<Self> {
        Arc::new(Self {
            server: Mutex::new(server),
            clients: Mutex::new(HashMap::new()),
            player_count,
            next_player_id: Mutex::new(0),
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = parse_arg(&args, "--port").unwrap_or(9876);
    let seed: u64 = parse_arg(&args, "--seed").unwrap_or(42);
    let player_count: u8 = parse_arg(&args, "--players").unwrap_or(2);

    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let server = RelayServer::new(
        1, 1, seed, 0, (0..player_count).collect(), 3, now_ms,
    );

    let ctx = RelayContext::new(server, player_count);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind TCP listener");

    println!("Relay on port {} (players={}, seed={})", port, player_count, seed);

    while let Ok((stream, addr)) = listener.accept().await {
        println!("Connect from {}", addr);
        tokio::spawn(handle(Arc::clone(&ctx), stream));
    }
}

async fn handle(ctx: Arc<RelayContext>, mut stream: tokio::net::TcpStream) {
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
    {
        let mut w = writer;
        write_frame(&mut w, &msg).await;

        // Writer task: forward all messages to TCP
        let write_h = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                write_frame(&mut w, &msg).await;
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

                    let batch = {
                        let mut server = ctx.server.lock().unwrap();
                        server.on_player_frame(&frame, now_ms)
                    };

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
    }

    {
        let mut server = ctx.server.lock().unwrap();
        server.on_disconnect(player_id);
    }
}

/// Write a length-prefixed bincode frame to the TCP stream.
async fn write_frame(
    writer: &mut (impl tokio::io::AsyncWrite + Unpin),
    msg: &RelayServerMessage,
) {
    if let Ok(data) = bincode::serde::encode_to_vec(msg, bincode::config::standard()) {
        let len_bytes = (data.len() as u32).to_le_bytes();
        let _ = writer.write_all(&len_bytes).await;
        let _ = writer.write_all(&data).await;
    }
}

fn parse_arg<T: std::str::FromStr>(args: &[String], name: &str) -> Option<T> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
}

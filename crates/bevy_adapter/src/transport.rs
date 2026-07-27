//! Client-side transport for RTS CommandStream Protocol v1.0.
//!
//! Cross-thread bridge between Bevy's main thread and a tokio async network runtime.

use bevy::prelude::*;

use crate::discovery::{RelayId, RoomId};
use crate::driver::CommandSource;
use crate::network::{
    BroadcastFrame, NetworkEvent, NetworkEventReceiver, PlayerTickFrame, RelayClientMessage,
    RelayServerMessage,
};
use simulation::command::{CommandBuffer, GameCommand};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

// ═══════════════════════════════════════════════════════════════
// Cross-thread channels

/// Inbound channel: tokio thread writes BroadcastFrames; Bevy system drains.
#[derive(Clone, Resource, Default)]
pub struct NetworkReceiver {
    inner: Arc<Mutex<VecDeque<BroadcastFrame>>>,
}

impl NetworkReceiver {
    /// Called by tokio thread to enqueue a received broadcast.
    pub fn push(&self, frame: BroadcastFrame) {
        self.inner.lock().unwrap().push_back(frame);
    }

    /// Drain all pending frames (called from Bevy system once per frame).
    pub fn drain_all(&self) -> Vec<BroadcastFrame> {
        let mut inner = self.inner.lock().unwrap();
        inner.drain(..).collect()
    }
}

/// Outbound channel: Bevy system queues PlayerTickFrames; tokio thread sends.
#[derive(Clone, Resource)]
pub struct NetworkSender {
    inner: Arc<Mutex<VecDeque<PlayerTickFrame>>>,
    next_sid: Arc<Mutex<u64>>,
    lobby_ready: Arc<Mutex<Option<RelayClientMessage>>>,

}

impl Default for NetworkSender {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            next_sid: Arc::new(Mutex::new(0)),
            lobby_ready: Arc::new(Mutex::new(None)),

        }
    }
}

impl NetworkSender {
    pub fn push(&self, frame: PlayerTickFrame) {
        self.inner.lock().unwrap().push_back(frame);
    }

    pub fn drain_all(&self) -> Vec<PlayerTickFrame> {
        let mut inner = self.inner.lock().unwrap();
        inner.drain(..).collect()
    }

    pub fn next_sid(&self) -> u64 {
        let mut sid = self.next_sid.lock().unwrap();
        *sid += 1;
        *sid
    }

    pub fn send_lobby_ready(&self, player_id: u8, ready: bool) {
        *self.lobby_ready.lock().unwrap() = Some(RelayClientMessage::LobbyReady {
            game_id: 1, player_id, ready, map_size: None,
        });
    }

    pub fn take_lobby_ready(&self) -> Option<RelayClientMessage> {
        self.lobby_ready.lock().unwrap().take()
    }
}

// ═══════════════════════════════════════════════════════════════
// Bevy systems
// ═══════════════════════════════════════════════════════════════

/// Poll the NetworkReceiver and push incoming BroadcastFrames to the active
/// NetworkCommandSource's relay_buffer. Must run BEFORE SimulationTickSet.
pub fn network_poll_system(
    receiver: Option<Res<NetworkReceiver>>,
    mut driver: ResMut<crate::driver::SimulationDriver>,
) {
    let receiver = match receiver {
        Some(r) => r,
        None => return,
    };
    let frames = receiver.drain_all();
    if frames.is_empty() {
        return;
    }
    bevy::log::info!("[NET] poll: received {} broadcast frames", frames.len());
    if let CommandSource::Network(ref mut ns) = driver.source {
        for frame in frames {
            let tick = frame.payload.tick;
            ns.push_broadcast(frame);
            bevy::log::info!("[NET] poll: pushed tick {} to relay_buffer", tick);
        }
    }
}

/// Flush local commands from Bevy cmd_buf to the NetworkSender.
/// This reads commands targeting the next input_delay'd tick and enqueues
/// them for transit to the relay.
pub fn network_flush_system(
    sender: Option<Res<NetworkSender>>,
    driver: Res<crate::driver::SimulationDriver>,
    cmd_buf: Res<CommandBuffer>,
) {
    let sender = match sender {
        Some(s) => s,
        None => return,
    };
    if let CommandSource::Network(ref ns) = driver.source {
        let current_tick = driver.clock.current_tick;
        // Send PlayerTickFrame for the NEXT tick the relay expects
        // Use current_tick + 1 (not delayed) so the relay processes the right tick
        let cmd_tick = current_tick + 1;

        // Always send a PlayerTickFrame even with empty commands.
        // The relay needs an empty frame to know the player is connected.
        let cmds: Vec<GameCommand> = cmd_buf
            .0
            .iter()
            .filter(|c| c.tick == current_tick + 1)
            .cloned()
            .collect();

        let sid = sender.next_sid();
        let frame = PlayerTickFrame {
            magic: 0xBEEF,
            version: 1,

            game_id: ns.game_id,
            tick: cmd_tick,
            player_id: ns.player_id,
            commands: cmds,
            player_sid: sid,
        };
        sender.push(frame);
    }
}


// ═══════════════════════════════════════════════════════════════
// Network client thread
// ═══════════════════════════════════════════════════════════════

/// Send RelayClientMessage::JoinGame over the established TCP stream.
/// Must be called after TCP connect, before run_session.
/// Returns Ok(()) on success, Err(String) on failure.
async fn send_join_game(
    stream: &mut tokio::net::TcpStream,
    relay_id: RelayId,
) -> Result<(), String> {
    let join_msg = RelayClientMessage::JoinGame {
        room_id: RoomId(0),
        relay_id,
    };
    let data = bincode::serde::encode_to_vec(&join_msg, bincode::config::standard())
        .map_err(|e| format!("JoinGame encode failed: {}", e))?;
    let len_bytes = (data.len() as u32).to_le_bytes();
    stream.write_all(&len_bytes).await
        .map_err(|e| format!("JoinGame write len failed: {}", e))?;
    stream.write_all(&data).await
        .map_err(|e| format!("JoinGame write data failed: {}", e))?;
    eprintln!("[NET] JoinGame sent (relay_id={:?}, {} bytes)", relay_id, data.len());
    Ok(())
}

/// Spawn a tokio runtime thread that connects to the relay.
///
/// **Blocks** until TCP connection is established (30s timeout).
/// Returns transport resources or an error string.
///
/// The tokio thread continues to run in the background, handling
/// all protocol communication (send/receive) until stopped.
pub fn spawn_network_client(
    relay_addr: String,
    game_id: u64,
    player_id: u8,
    _ruleset_version: u32,
    event_receiver: NetworkEventReceiver,
    relay_id: RelayId,
) -> Result<(NetworkReceiver, NetworkSender, NetworkClientHandle), String> {
    let receiver = NetworkReceiver::default();
    let sender = NetworkSender::default();
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let (connected_tx, connected_rx) = std::sync::mpsc::channel::<()>();

    let recv = receiver.clone();
    let send = sender.clone();
    let stop = stop_flag.clone();
    let events = event_receiver;
    let relay_addr_for_thread = relay_addr.clone();

    let thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("Failed to build tokio runtime");

        rt.block_on(async move {
            let mut retry_count = 0u32;
            loop {
                if stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                // Try TCP connect with 5s timeout
                let stream = tokio::time::timeout(
                    Duration::from_secs(5),
                    tokio::net::TcpStream::connect(&relay_addr_for_thread),
                )
                .await;

                let mut stream = match stream {
                    Ok(Ok(s)) => s,
                    _ => {
                        retry_count = retry_count.saturating_add(1);
                        let delay = Duration::from_secs(
                            (1u64 << retry_count.min(5)).min(30),
                        );
                        eprintln!(
                            "Network: retrying in {}s (attempt {})",
                            delay.as_secs(),
                            retry_count
                        );
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                };

                // TCP connected! Signal the main thread (spawn_network_client unblocks)
                let _ = connected_tx.send(());
                retry_count = 0;

                // Send JoinGame before entering session loop
                if let Err(e) = send_join_game(&mut stream, relay_id).await {
                    eprintln!("[NET] JoinGame failed: {} — retrying", e);
                    continue;
                }

                // Enter read/write session
                let clean_exit = run_session(
                    stream,
                    game_id,
                    player_id,
                    _ruleset_version,
                    &recv,
                    &send,
                    stop.clone(),
                    &events,
                )
                .await;

                if clean_exit {
                    break;
                }
                // If not clean (disconnect), retry the entire connection
            }
        });
    });

    // Block until TCP connect succeeds (30s timeout)
    connected_rx
        .recv_timeout(Duration::from_secs(30))
        .map_err(|_| format!("Timed out waiting for relay connection at {}", relay_addr))?;
    eprintln!("[NET] TCP connected to {}", relay_addr);

    let handle = NetworkClientHandle {
        thread: Some(thread),
        stop: stop_flag,
    };

    Ok((receiver, sender, handle))
}

/// 非阻塞变体：启动 TCP 连接后立即返回，连接状态通过 `LobbyConnectionStatus` 轮询。
pub fn spawn_network_client_nonblocking(
    relay_addr: String,
    game_id: u64,
    player_id: u8,
    _ruleset_version: u32,
    event_receiver: NetworkEventReceiver,
    relay_id: RelayId,
) -> (NetworkReceiver, NetworkSender, NetworkClientHandle, LobbyConnectionStatus) {
    let receiver = NetworkReceiver::default();
    let sender = NetworkSender::default();
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let status = LobbyConnectionStatus::default();

    let recv = receiver.clone();
    let send = sender.clone();
    let stop = stop_flag.clone();
    let events = event_receiver;
    let relay_addr_for_thread = relay_addr.clone();
    let conn_status = status.clone();

    let thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("Failed to build tokio runtime");

        rt.block_on(async move {
            let mut retry_count = 0u32;
            loop {
                if stop.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                let stream = tokio::time::timeout(
                    Duration::from_secs(5),
                    tokio::net::TcpStream::connect(&relay_addr_for_thread),
                )
                .await;

                let mut stream = match stream {
                    Ok(Ok(s)) => s,
                    _ => {
                        retry_count = retry_count.saturating_add(1);
                        let delay = Duration::from_secs((1u64 << retry_count.min(5)).min(30));
                        eprintln!("Network: retrying in {}s (attempt {})", delay.as_secs(), retry_count);
                        // After 8+ retries (total elapsed ~8.5 minutes), signal failure
                        if retry_count >= 8 {
                            conn_status.result.lock().unwrap().replace(
                                Err(format!("Failed to connect after {} attempts", retry_count))
                            );
                        }
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                };

                // TCP connected! Signal via status channel
                conn_status.result.lock().unwrap().replace(Ok(()));
                retry_count = 0;

                // Send JoinGame before entering session loop
                if let Err(e) = send_join_game(&mut stream, relay_id).await {
                    eprintln!("[NET] JoinGame failed: {} — retrying", e);
                    continue;
                }

                let clean_exit = run_session(
                    stream, game_id, player_id, _ruleset_version,
                    &recv, &send, stop.clone(), &events,
                ).await;

                if clean_exit { break; }
            }
        });
    });

    let handle = NetworkClientHandle { thread: Some(thread), stop: stop_flag };
    (receiver, sender, handle, status)
}

/// Run a single client session over an already-established TCP stream.
/// Returns `true` on clean disconnect (game over), `false` on connection error.
async fn run_session(
    stream: tokio::net::TcpStream,
    game_id: u64,
    player_id: u8,
    _ruleset_version: u32,
    receiver: &NetworkReceiver,
    sender: &NetworkSender,
    stop: Arc<std::sync::atomic::AtomicBool>,
    event_receiver: &NetworkEventReceiver,
) -> bool {
    let (mut reader, mut writer) = tokio::io::split(stream);

    // Write frames to relay in the background
    let send = sender.clone();
    let stop_arc = stop.clone();
    let write_task = tokio::spawn(async move {
        loop {
            if stop_arc.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            // Check for lobby-ready (one-shot) message before game frames
            if let Some(lobby_msg) = send.take_lobby_ready() {
                if let Ok(data) = bincode::serde::encode_to_vec(&lobby_msg, bincode::config::standard()) {
                    let len_bytes = (data.len() as u32).to_le_bytes();
                    let _ = writer.write_all(&len_bytes).await;
                    let _ = writer.write_all(&data).await;
                }
            }
            let frames = send.drain_all();
            for frame in frames {
                let msg = RelayClientMessage::PlayerTick(frame);
                if let Ok(data) = bincode::serde::encode_to_vec(&msg, bincode::config::standard()) {
                    let len_bytes = (data.len() as u32).to_le_bytes();
                    let _ = writer.write_all(&len_bytes).await;
                    let _ = writer.write_all(&data).await;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    // Read frames from relay
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
        let Ok((msg, _)) = bincode::serde::decode_from_slice::<RelayServerMessage, _>(
            &buf,
            bincode::config::standard(),
        ) else {
            continue;
        };
        match msg {
            RelayServerMessage::Broadcast(frame) => {
                receiver.push(frame);
            }
            RelayServerMessage::GameStarted {
                game_id: g,
                seed,
                player_count,
            } => {
                eprintln!(
                    "[NET] GameStarted: game_id={}, seed={}, players={}",
                    g, seed, player_count
                );
                event_receiver.push(NetworkEvent::GameStarted {
                    game_id: g,
                    seed,
                    player_count,
                });
            }
            RelayServerMessage::GameJoined {
                game_id: g,
                player_id: p,
                player_count,
            } => {
                eprintln!("[NET] Joined game {} as player {} (of {})", g, p, player_count);
                event_receiver.push(NetworkEvent::GameJoined {
                    player_id: p,
                    player_count,
                });
            }
            RelayServerMessage::ReconnectResponse(resp) => {
                eprintln!("[NET] Reconnect OK ({} ticks)", resp.ticks.len());
            }
            RelayServerMessage::GameOver { reason } => {
                eprintln!("[NET] Game over ({})", reason);
                write_task.abort();
                return true;
            }
            RelayServerMessage::Error { code, message } => {
                eprintln!("[NET] Error ({}): {}", code, message);
                write_task.abort();
                return true;
            }
            RelayServerMessage::LobbyUpdate { game_id, players } => {
                event_receiver.push(NetworkEvent::LobbyUpdate { game_id, players });
            }
            RelayServerMessage::JoinRejected { reason } => {
                eprintln!("[NET] Join rejected: {}", reason);
                write_task.abort();
                return true;
            }
        }
    }

    write_task.abort();
    false
}

/// 跨线程连接状态，用于 Lobby 非阻塞轮询 TCP 连接进度。
#[derive(Clone, Default)]
pub struct LobbyConnectionStatus {
    pub result: Arc<Mutex<Option<Result<(), String>>>>,
}

impl LobbyConnectionStatus {
    /// 检查并消费连接结果。返回 None 表示仍在连接中。
    pub fn poll(&self) -> Option<Result<(), String>> {
        self.result.lock().unwrap().take()
    }

    /// 获取内部 Arc 引用（用于构造 ConnectionPollRx 等）
    pub fn inner_arc(&self) -> Arc<Mutex<Option<Result<(), String>>>> {
        self.result.clone()
    }
}

/// Handle that stops the network thread when dropped.
#[derive(Resource)]
pub struct NetworkClientHandle {
    thread: Option<std::thread::JoinHandle<()>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
}

impl NetworkClientHandle {
    /// Forcefully stop the network thread.
    pub fn abort(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Drop for NetworkClientHandle {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

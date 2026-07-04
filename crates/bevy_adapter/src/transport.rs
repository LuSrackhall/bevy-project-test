//! Client-side transport for RTS CommandStream Protocol v1.0.
//!
//! Cross-thread bridge between Bevy's main thread and a tokio async network runtime.

use bevy::prelude::*;

use crate::driver::{CommandSource};
use crate::network::{
    BroadcastFrame, PlayerTickFrame, RelayClientMessage,
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
// ═══════════════════════════════════════════════════════════════

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
}

impl Default for NetworkSender {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            next_sid: Arc::new(Mutex::new(0)),
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
        None => return, // Not in Network mode — no transport resources
    };
    let frames = receiver.drain_all();
    if frames.is_empty() {
        return;
    }
    if let CommandSource::Network(ref mut ns) = driver.source {
        for frame in frames {
            ns.push_broadcast(frame);
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
        let target_tick = ns.delayed_tick(current_tick);

        // Collect commands for the delayed tick
        let cmds: Vec<GameCommand> = cmd_buf
            .0
            .iter()
            .filter(|c| c.tick == target_tick)
            .cloned()
            .collect();

        if cmds.is_empty() {
            return;
        }

        let sid = sender.next_sid();
        let frame = PlayerTickFrame {
            magic: 0xBEEF,
            game_id: ns.game_id,
            tick: target_tick,
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

/// Spawn a tokio runtime thread that connects to the relay.
pub fn spawn_network_client(
    relay_addr: String,
    game_id: u64,
    player_id: u8,
    _ruleset_version: u32,
) -> (NetworkReceiver, NetworkSender, NetworkClientHandle) {
    let receiver = NetworkReceiver::default();
    let sender = NetworkSender::default();
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let recv = receiver.clone();
    let send = sender.clone();
    let stop = stop_flag.clone();

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

                let ok = tokio::time::timeout(
                    Duration::from_secs(5),
                    run_client(&relay_addr, game_id, player_id, _ruleset_version, &recv, &send, stop.clone()),
                )
                .await;

                match ok {
                    Ok(true) => {
                        retry_count = 0;
                        // Clean disconnect (game over) — stop
                        break;
                    }
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
                    }
                }
            }
        });
    });

    let handle = NetworkClientHandle {
        thread: Some(thread),
        stop: stop_flag,
    };

    (receiver, sender, handle)
}

/// Spawn network client with GameJoined return channel for bootstrap handshake.
pub fn spawn_with_game_joined(
    relay_addr: &str,
    game_id: u64,
) -> Result<
    (
        std::sync::mpsc::Receiver<u8>,
        NetworkReceiver,
        NetworkSender,
        NetworkClientHandle,
    ),
    String,
> {
    // For Phase 1, delegate to the standard spawn and return a dummy channel.
    // The GameJoined player_id is currently assigned sequentially by the relay
    // and known to the client via spawn order. A full implementation would
    // extract the player_id from the transport's GameJoined message.
    let player_id = 0; // placeholder; real value comes from relay handshake
    let (rx, tx, handle) = spawn_network_client(
        relay_addr.to_string(),
        game_id,
        player_id,
        1,
    );
    // Use oneshot channel for GameJoined delivery
    let (game_joined_tx, game_joined_rx) = std::sync::mpsc::channel();
    // In a full implementation, transport's run_client would send() on
    // game_joined_tx when GameJoined is received. For Phase 1, send once
    // we know the connection succeeded.
    let _ = game_joined_tx.send(player_id);
    Ok((game_joined_rx, rx, tx, handle))
}

/// Run a single client connection session. Returns true on clean disconnect.
async fn run_client(
    addr: &str,
    game_id: u64,
    player_id: u8,
    _ruleset_version: u32,
    receiver: &NetworkReceiver,
    sender: &NetworkSender,
    stop: Arc<std::sync::atomic::AtomicBool>,
) -> bool {
    let stream = match tokio::net::TcpStream::connect(addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Network: connect failed: {}", e);
            return false;
        }
    };
    let (mut reader, mut writer) = tokio::io::split(stream);

    // Write frames to relay in the background
    let send = sender.clone();
    let stop_arc = stop.clone();
    let write_task = tokio::spawn(async move {
        loop {
            if stop_arc.load(std::sync::atomic::Ordering::Relaxed) {
                break;
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
        if reader.read_exact(&mut len_buf)
            .await
            .is_err()
        {
            break;
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        if reader.read_exact(&mut buf)
            .await
            .is_err()
        {
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
            RelayServerMessage::GameJoined {
                game_id: g,
                player_id: p,
            } => {
                println!("Network: joined game {} as player {}", g, p);
            }
            RelayServerMessage::ReconnectResponse(resp) => {
                println!("Network: reconnect OK ({} ticks)", resp.ticks.len());
            }
            RelayServerMessage::GameOver { reason } => {
                println!("Network: game over ({})", reason);
                write_task.abort();
                return true;
            }
            RelayServerMessage::Error { code, message } => {
                eprintln!("Network error ({}): {}", code, message);
                write_task.abort();
                return true;
            }
        }
    }

    write_task.abort();
    false
}

/// Handle that stops the network thread when dropped.
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

//! Client-side transport for RTS CommandStream Protocol v1.0.
//!
//! Cross-thread bridge between Bevy's main thread and a tokio async network
//! runtime running the self-written reliable UDP transport.

use bevy::prelude::*;

use crate::discovery::{RelayId, RoomId};
use crate::driver::CommandSource;
use crate::network::{
    BroadcastFrame, NetworkEvent, NetworkEventReceiver, PlayerTickFrame, ReconnectRequest,
    RelayClientMessage, RelayServerMessage,
};
use crate::reliable_udp::channel_udp::UdpChannel;
use crate::reliable_udp::protocol::{CH_CONTROL, CH_TICK};
use crate::reliable_udp::{ReliableConfig, ReliableSocket};
use simulation::command::{CommandBuffer, GameCommand};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    /// Last tick the local simulation consumed. Used as last_tick_consumed on reconnect.
    last_tick_consumed: Arc<Mutex<u32>>,
}

impl Default for NetworkSender {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            next_sid: Arc::new(Mutex::new(0)),
            lobby_ready: Arc::new(Mutex::new(None)),
            last_tick_consumed: Arc::new(Mutex::new(0)),
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

    /// Bevy 侧更新本地已消费到的 tick,供重连时作为 last_tick_consumed。
    pub fn update_last_tick_consumed(&self, tick: u32) {
        *self.last_tick_consumed.lock().unwrap() = tick;
    }

    /// 当前已消费 tick(重连断点)。
    pub fn last_tick_consumed(&self) -> u32 {
        *self.last_tick_consumed.lock().unwrap()
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
pub fn network_flush_system(
    sender: Option<Res<NetworkSender>>,
    driver: Res<crate::driver::SimulationDriver>,
    mut cmd_buf: ResMut<CommandBuffer>,
) {
    let sender = match sender {
        Some(s) => s,
        None => return,
    };
    if let CommandSource::Network(ref ns) = driver.source {
        let current_tick = driver.clock.current_tick;
        // 同步本地已消费 tick,供重连时作为 last_tick_consumed
        sender.update_last_tick_consumed(current_tick);
        // Send frames for the ENTIRE window [current_tick+1, current_tick+input_delay].
        let start = current_tick + 1;
        let end = ns.delayed_tick(current_tick);
        for tick in start..=end {
            let cmds = cmd_buf.take_for_tick(tick);
            let sid = sender.next_sid();
            let frame = PlayerTickFrame {
                magic: 0xBEEF,
                version: 1,
                game_id: ns.game_id,
                tick,
                player_id: ns.player_id,
                commands: cmds,
                player_sid: sid,
            };
            sender.push(frame);
        }
    }
}

/// Process `NetworkEvent::Reconnect` — apply the relay's command log so the
/// driver resumes from the disconnect point. Runs BEFORE SimulationTickSet.
pub fn reconnect_recovery_system(
    event_receiver: Option<Res<NetworkEventReceiver>>,
    mut driver: ResMut<crate::driver::SimulationDriver>,
) {
    let Some(receiver) = event_receiver else { return };
    let events = receiver.drain_all();
    for event in events {
        // 重连后 relay 重新分配 player_id——必须更新,否则命令归属错误 → desync
        if let NetworkEvent::GameJoined { player_id, .. } = event {
            if let CommandSource::Network(ref mut ns) = driver.source {
                ns.player_id = player_id;
                eprintln!("[NET] reconnect: player_id updated to {}", player_id);
            }
        }
        if let NetworkEvent::Reconnect(resp) = event {
            if let CommandSource::Network(ref mut ns) = driver.source {
                // 规则版本当前全局硬编码 1
                let expected = 1u32;
                match ns.apply_reconnect(&resp, expected) {
                    Ok(()) => eprintln!(
                        "[NET] reconnect applied ({} ticks, first={})",
                        resp.ticks.len(),
                        resp.first_tick
                    ),
                    Err(e) => eprintln!("[NET] reconnect apply failed: {}", e),
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Network client thread (reliable UDP)
// ═══════════════════════════════════════════════════════════════

/// Client UDP session: connect to relay, send JoinGame, then pump the reliable
/// socket. Returns `true` on clean exit (game over / rejected), `false` when the
/// connection must be retried (reconnect).
async fn udp_session(
    relay_addr: String,
    game_id: u64,
    player_id: u8,
    _ruleset_version: u32,
    receiver: &NetworkReceiver,
    sender: &NetworkSender,
    stop: Arc<std::sync::atomic::AtomicBool>,
    event_receiver: &NetworkEventReceiver,
    on_joined: Option<&std::sync::mpsc::Sender<()>>,
    conn_status: Option<&LobbyConnectionStatus>,
    relay_id: RelayId,
) -> bool {
    let peer: SocketAddr = match relay_addr.parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    let sock = match UdpChannel::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut rs = ReliableSocket::new(Box::new(sock), peer, ReliableConfig::default());
    let start = std::time::Instant::now();

    // JoinGame on the Control channel (reliable).
    let join = RelayClientMessage::JoinGame { room_id: RoomId(0), relay_id };
    if let Ok(data) = bincode::serde::encode_to_vec(&join, bincode::config::standard()) {
        rs.send_reliable(CH_CONTROL, data);
    }
    // Reconnect: if we consumed ticks before, request the log from the disconnect point.
    if sender.last_tick_consumed() > 0 {
        let req = RelayClientMessage::Reconnect(ReconnectRequest {
            game_id,
            last_tick_consumed: sender.last_tick_consumed(),
        });
        if let Ok(data) = bincode::serde::encode_to_vec(&req, bincode::config::standard()) {
            rs.send_reliable(CH_CONTROL, data);
        }
    }

    let mut last_activity = std::time::Instant::now();
    let mut last_heartbeat = std::time::Instant::now();

    loop {
        if stop.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }
        rs.set_now(start.elapsed());
        rs.process();
        if rs.poll().await.is_err() {
            return false;
        }
        for msg in rs.take_messages() {
            if let Ok((server_msg, _)) = bincode::serde::decode_from_slice::<RelayServerMessage, _>(
                &msg,
                bincode::config::standard(),
            ) {
                last_activity = std::time::Instant::now();
                match server_msg {
                    RelayServerMessage::Broadcast(frame) => receiver.push(frame),
                    RelayServerMessage::GameStarted { game_id: g, seed, player_count } => {
                        eprintln!("[NET] GameStarted: game_id={}, seed={}, players={}", g, seed, player_count);
                        event_receiver.push(NetworkEvent::GameStarted { game_id: g, seed, player_count });
                    }
                    RelayServerMessage::GameJoined { game_id: g, player_id: p, player_count } => {
                        eprintln!("[NET] Joined game {} as player {} (of {})", g, p, player_count);
                        event_receiver.push(NetworkEvent::GameJoined { player_id: p, player_count });
                        if let Some(tx) = on_joined {
                            let _ = tx.send(());
                        }
                        if let Some(status) = conn_status {
                            status.result.lock().unwrap().replace(Ok(()));
                        }
                    }
                    RelayServerMessage::ReconnectResponse(resp) => {
                        eprintln!("[NET] Reconnect OK ({} ticks)", resp.ticks.len());
                        event_receiver.push(NetworkEvent::Reconnect(resp));
                    }
                    RelayServerMessage::LobbyUpdate { game_id, players } => {
                        event_receiver.push(NetworkEvent::LobbyUpdate { game_id, players });
                    }
                    RelayServerMessage::GameOver { reason } => {
                        eprintln!("[NET] Game over ({})", reason);
                        return true;
                    }
                    RelayServerMessage::Error { code, message } => {
                        eprintln!("[NET] Error ({}): {}", code, message);
                        return true;
                    }
                    RelayServerMessage::JoinRejected { reason } => {
                        eprintln!("[NET] Join rejected: {}", reason);
                        // 重连场景被拒 → 重试;首次 join 被拒 → 放弃
                        return sender.last_tick_consumed() == 0;
                    }
                }
            }
        }

        // Uplink: lobby-ready + PlayerTick commands.
        if let Some(lobby_msg) = sender.take_lobby_ready() {
            if let Ok(data) = bincode::serde::encode_to_vec(&lobby_msg, bincode::config::standard()) {
                rs.send_reliable(CH_CONTROL, data);
            }
        }
        for frame in sender.drain_all() {
            let msg = RelayClientMessage::PlayerTick(frame);
            if let Ok(data) = bincode::serde::encode_to_vec(&msg, bincode::config::standard()) {
                rs.send_reliable(CH_TICK, data);
            }
        }

        // Heartbeat every 500ms (keeps the relay's session alive).
        if last_heartbeat.elapsed() >= Duration::from_millis(500) {
            rs.send_unreliable(vec![]);
            last_heartbeat = std::time::Instant::now();
        }

        // Disconnect detection: no relay message for 3s → reconnect.
        if last_activity.elapsed() >= Duration::from_secs(3) {
            eprintln!("[NET] connection timeout — reconnecting");
            return false;
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Spawn a tokio runtime thread that connects to the relay over UDP.
///
/// **Blocks** until GameJoined is received (30s timeout).
/// Returns transport resources or an error string.
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
                let clean_exit = udp_session(
                    relay_addr_for_thread.clone(),
                    game_id,
                    player_id,
                    _ruleset_version,
                    &recv,
                    &send,
                    stop.clone(),
                    &events,
                    Some(&connected_tx),
                    None,
                    relay_id,
                )
                .await;

                if clean_exit {
                    break;
                }
                retry_count = retry_count.saturating_add(1);
                let delay = Duration::from_secs((1u64 << retry_count.min(5)).min(30));
                eprintln!("[NET] reconnecting in {}s (attempt {})", delay.as_secs(), retry_count);
                tokio::time::sleep(delay).await;
            }
        });
    });

    // Block until GameJoined (30s timeout).
    connected_rx
        .recv_timeout(Duration::from_secs(30))
        .map_err(|_| format!("Timed out waiting for relay GameJoined at {}", relay_addr))?;
    eprintln!("[NET] Connected to {}", relay_addr);

    let handle = NetworkClientHandle {
        thread: Some(thread),
        stop: stop_flag,
    };

    Ok((receiver, sender, handle))
}

/// 非阻塞变体:启动 UDP 连接后立即返回,连接状态通过 `LobbyConnectionStatus` 轮询。
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
                let clean_exit = udp_session(
                    relay_addr_for_thread.clone(),
                    game_id,
                    player_id,
                    _ruleset_version,
                    &recv,
                    &send,
                    stop.clone(),
                    &events,
                    None,
                    Some(&conn_status),
                    relay_id,
                )
                .await;

                if clean_exit {
                    break;
                }
                retry_count = retry_count.saturating_add(1);
                let delay = Duration::from_secs((1u64 << retry_count.min(5)).min(30));
                eprintln!("[NET] reconnecting in {}s (attempt {})", delay.as_secs(), retry_count);
                tokio::time::sleep(delay).await;
            }
        });
    });

    let handle = NetworkClientHandle { thread: Some(thread), stop: stop_flag };
    (receiver, sender, handle, status)
}

/// 跨线程连接状态,用于 Lobby 非阻塞轮询连接进度。
#[derive(Clone, Default)]
pub struct LobbyConnectionStatus {
    pub result: Arc<Mutex<Option<Result<(), String>>>>,
}

impl LobbyConnectionStatus {
    /// 检查并消费连接结果。返回 None 表示仍在连接中。
    pub fn poll(&self) -> Option<Result<(), String>> {
        self.result.lock().unwrap().take()
    }

    /// 获取内部 Arc 引用(用于构造 ConnectionPollRx 等)
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
        // 不 join:网络线程可能阻塞在 poll,join 会死等。正常结束时线程自行退出。
        self.thread.take();
    }
}

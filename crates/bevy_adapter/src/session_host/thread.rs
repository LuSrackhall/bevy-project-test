use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use tokio::net::UdpSocket;

use crate::discovery::{LanDiscoveryPacket, RelayId, RoomAdvertisement, RoomMetadata};
use crate::relay_core::{self, RelayConfig};

use super::error::RelayError;
use super::runtime::{RelayHandle, RelayRuntime};

/// RelayRuntime implementation that starts relay in a background thread
/// with its own tokio runtime.
pub struct ThreadRelayRuntime;

impl RelayRuntime for ThreadRelayRuntime {
    fn start(&mut self, room: &RoomMetadata) -> Result<Box<dyn RelayHandle>, RelayError> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_inner = stop.clone();
        let stop_for_thread = stop.clone();

        // Generate relay_id BEFORE spawning the thread so it's shared between
        // the beacon (for dedup by relay_id) and the RelayHandle (for UI matching).
        let relay_id = RelayId(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42),
        );

        let room_clone = room.clone();

        // Channel for thread to report the actual bound port (or error)
        let (port_tx, port_rx) = mpsc::channel();

        let handle = thread::Builder::new()
            .name("relay-host".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_io()
                    .enable_time()
                    .build()
                    .expect("Failed to build relay tokio runtime");

                rt.block_on(async move {
                    run_local_relay(&port_tx, relay_id, &room_clone, &stop_for_thread).await;
                });
            })
            .map_err(|e| RelayError::StartFailed(format!("Thread spawn failed: {}", e)))?;

        // Wait for the thread to bind and report the port
        let actual_port = port_rx
            .recv()
            .map_err(|_| RelayError::StartFailed("Thread died before binding".into()))??;

        Ok(Box::new(ThreadRelayHandle {
            relay_id,
            endpoint: SocketAddr::from(([127, 0, 0, 1], actual_port)),
            stop: stop_inner,
            handle: Mutex::new(Some(handle)),
        }))
    }
}

/// Run the full local relay: TCP bind, UDP beacon, shared relay loop.
async fn run_local_relay(
    port_tx: &mpsc::Sender<Result<u16, RelayError>>,
    relay_id: RelayId,
    room: &RoomMetadata,
    stop: &Arc<AtomicBool>,
) {
    // Live connected-client count, shared between the beacon (for current_players)
    // and the relay (updated on join/disconnect).
    let clients_count = Arc::new(AtomicUsize::new(0));

    // Bind dual-stack UDP socket — accept connections over IPv4/IPv6, OS allocates port
    let socket = match UdpSocket::bind("[::]:0").await {
        Ok(s) => s,
        Err(e) => {
            let _ = port_tx.send(Err(RelayError::StartFailed(format!(
                "UDP bind failed: {}",
                e
            ))));
            return;
        }
    };

    let actual_port = socket.local_addr().map(|a| a.port()).unwrap_or(9876);
    let _ = port_tx.send(Ok(actual_port));

    // Discovery beacon binds an EPHEMERAL port (0.0.0.0:0) and broadcasts TO
    // :9876. It must NOT bind 9876: the `LanDiscoveryListener` (active while
    // browsing the room list) already holds 0.0.0.0:9876, so a 9876 beacon bind
    // fails with EADDRINUSE and silently disables discovery. The listener
    // matches rooms by the packet's `relay_id`, not the source port.
    let udp_socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => {
            if let Err(e) = s.set_broadcast(true) {
                eprintln!("[BEACON] set_broadcast(true) failed: {}", e);
            }
            Some(s)
        }
        Err(e) => {
            eprintln!("[BEACON] UDP bind failed (beacon disabled): {}", e);
            None
        }
    };

    // Detect LAN IP so remote clients can connect (not 127.0.0.1)
    let lan_ip = detect_lan_ip();
    eprintln!("[RELAY] LAN IP: {}, port: {}", lan_ip, actual_port);

    // Derive a /24 subnet broadcast address (e.g. 192.168.1.157 → 192.168.1.255).
    // Some Windows adapters do not forward the 255.255.255.255 global broadcast,
    // so sending to the subnet broadcast improves cross-machine discovery.
    let subnet_broadcast = match lan_ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            Some(format!("{}.{}.{}.255", octets[0], octets[1], octets[2]))
        }
        std::net::IpAddr::V6(_) => None,
    };
    if let Some(bc) = &subnet_broadcast {
        eprintln!("[RELAY] Subnet broadcast: {}:9876", bc);
    }

    // Spawn beacon as a separate task on this runtime so it runs
    // concurrently with the relay accept loop.
    if let Some(socket) = udp_socket {
        let stop_beacon = Arc::clone(stop);
        let mut beacon_room = room.clone();
        let beacon_endpoint = format!("{}:{}", lan_ip, actual_port);
        let beacon_clients = clients_count.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                if stop_beacon.load(Ordering::Relaxed) {
                    break;
                }
                interval.tick().await;
                // 实时连接数驱动 current_players(specs/multiplayer-scale)
                beacon_room.current_players = beacon_clients.load(Ordering::Relaxed) as u8;
                let pkt = LanDiscoveryPacket::new(RoomAdvertisement {
                    relay_id,
                    endpoint: beacon_endpoint.clone(),
                    room: beacon_room.clone(),
                });
                if let Ok(data) = pkt.encode() {
                    let _ = socket.send_to(&data, "255.255.255.255:9876").await;
                    if let Some(bc) = &subnet_broadcast {
                        let _ = socket.send_to(&data, format!("{}:9876", bc)).await;
                    }
                    let _ = socket.send_to(&data, "127.0.0.1:9876").await;
                }
            }
        });
    }

    // Delegate to shared relay runtime for UDP client handling
    let config = RelayConfig {
        relay_id,
        game_id: 1,
        ruleset_version: 1,
        seed: room.room_id.0,
        map_spec_hash: 0,
        // 网络路径当前统一 Medium(reset_game_system 硬编码);map_id→MapSize 解析留后续
        map_size: simulation::map::MapSize::Medium,
        player_count: room.max_players,
        input_delay: 3,
        current_clients: clients_count.clone(),
    };
    relay_core::run_relay(socket, config, stop).await;
}

/// Handle for a thread-based relay instance.
pub struct ThreadRelayHandle {
    relay_id: RelayId,
    endpoint: SocketAddr,
    stop: Arc<AtomicBool>,
    handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl RelayHandle for ThreadRelayHandle {
    fn relay_id(&self) -> RelayId {
        self.relay_id
    }

    fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    fn shutdown(self: Box<Self>) -> Result<(), RelayError> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.lock().unwrap().take() {
            h.join().map_err(|_| RelayError::ShutdownFailed("Thread join failed".into()))?;
        }
        Ok(())
    }
}

/// Detect this machine's LAN IP by opening a UDP socket to an external address.
/// Falls back to 127.0.0.1 if detection fails (e.g., no network).
fn detect_lan_ip() -> std::net::IpAddr {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:80").and_then(|_| s.local_addr()))
        .map(|a| a.ip())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
}

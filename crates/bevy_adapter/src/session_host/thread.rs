use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use tokio::net::{TcpListener, UdpSocket};

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
    // Bind to port 0 — OS allocates a free port
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(l) => l,
        Err(e) => {
            let _ = port_tx.send(Err(RelayError::StartFailed(format!(
                "TCP bind failed: {}",
                e
            ))));
            return;
        }
    };

    let actual_port = listener.local_addr().map(|a| a.port()).unwrap_or(9876);
    let _ = port_tx.send(Ok(actual_port));

    // Create UDP socket for LAN discovery beacon broadcasting.
    let udp_socket = match UdpSocket::bind(format!("0.0.0.0:{}", actual_port)).await {
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

    // Spawn beacon as a separate task on this runtime so it runs
    // concurrently with the relay accept loop.
    if let Some(socket) = udp_socket {
        let stop_beacon = Arc::clone(stop);
        let beacon_room = room.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                if stop_beacon.load(Ordering::Relaxed) {
                    break;
                }
                interval.tick().await;
                let pkt = LanDiscoveryPacket::new(RoomAdvertisement {
                    relay_id,
                    endpoint: format!("127.0.0.1:{}", actual_port),
                    room: beacon_room.clone(),
                });
                if let Ok(data) = pkt.encode() {
                    let _ = socket.send_to(&data, "255.255.255.255:9876").await;
                    let _ = socket.send_to(&data, "127.0.0.1:9876").await;
                }
            }
        });
    }

    // Delegate to shared relay runtime for TCP accept + client handling
    let config = RelayConfig {
        relay_id,
        game_id: 1,
        ruleset_version: 1,
        seed: room.room_id.0,
        map_spec_hash: 0,
        player_count: room.max_players,
        input_delay: 3,
    };
    relay_core::run_relay(listener, config, stop).await;
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

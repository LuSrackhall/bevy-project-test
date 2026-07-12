use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::SystemTime;

use tokio::net::TcpListener;

use crate::discovery::{RelayId, RoomMetadata};
use crate::network::RelayServer;

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

        let seed: u64 = room.room_id.0;
        let max_players = room.max_players;

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
                    run_local_relay(&port_tx, seed, max_players, &stop_for_thread).await;
                });
            })
            .map_err(|e| RelayError::StartFailed(format!("Thread spawn failed: {}", e)))?;

        // Wait for the thread to bind and report the port
        let actual_port = port_rx
            .recv()
            .map_err(|_| RelayError::StartFailed("Thread died before binding".into()))??;

        // 仅用于网络层元数据（连接跟踪/日志），不回流至 simulation，
        // 不影响确定性仿真（宪法 §2.6）。
        let relay_id = RelayId(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42),
        );

        Ok(Box::new(ThreadRelayHandle {
            relay_id,
            endpoint: SocketAddr::from(([127, 0, 0, 1], actual_port)),
            stop: stop_inner,
            handle: Mutex::new(Some(handle)),
        }))
    }
}

/// Run a minimal local relay on a background thread.
/// Binds to port 0 (OS allocation) and sends the actual port back.
/// Binds to port 0 (OS allocation) and sends the actual port back.
async fn run_local_relay(
    port_tx: &mpsc::Sender<Result<u16, RelayError>>,
    seed: u64,
    max_players: u8,
    stop: &AtomicBool,
) {
    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let server = RelayServer::new(
        1, 1, seed, 0, (0..max_players).collect(), 3, now_ms,
    );

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

    // Accept connections until stop signal
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        // Note: full relay accept loop is handled by relay crate.
        // For LAN MVP this starts the relay. A complete accept loop
        // will replace the placeholder once #8 integrates the join flow.
    }

    // Keep server alive to prevent drop
    let _ = server;
    let _ = listener;
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

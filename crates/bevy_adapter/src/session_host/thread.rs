use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use tokio::net::TcpListener;

use crate::discovery::{RelayId, RoomMetadata};
use crate::network::RelayServer;

use super::error::RelayError;
use super::runtime::{RelayHandle, RelayRuntime};

/// Default relay port for LAN sessions.
const DEFAULT_RELAY_PORT: u16 = 9876;

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

        let handle = thread::Builder::new()
            .name("relay-host".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_io()
                    .enable_time()
                    .build()
                    .expect("Failed to build relay tokio runtime");

                rt.block_on(async move {
                    run_local_relay(DEFAULT_RELAY_PORT, seed, max_players, &stop_for_thread).await;
                });
            })
            .map_err(|e| RelayError::StartFailed(format!("Thread spawn failed: {}", e)))?;

        let relay_id = RelayId(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42),
        );

        Ok(Box::new(ThreadRelayHandle {
            relay_id,
            endpoint: SocketAddr::from(([127, 0, 0, 1], DEFAULT_RELAY_PORT)),
            stop: stop_inner,
            handle: Some(handle),
        }))
    }
}

/// Run a minimal local relay on a background thread.
/// Simplified version of `relay::start_relay` that runs until stopped.
async fn run_local_relay(port: u16, seed: u64, max_players: u8, stop: &AtomicBool) {
    let now_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let server = RelayServer::new(
        1, 1, seed, 0, (0..max_players).collect(), 3, now_ms,
    );

    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[SessionHost] Failed to bind relay: {}", e);
            return;
        }
    };

    // Accept connections until stop signal
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
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
    handle: Option<thread::JoinHandle<()>>,
}

impl RelayHandle for ThreadRelayHandle {
    fn relay_id(&self) -> RelayId {
        self.relay_id
    }

    fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    fn shutdown(mut self: Box<Self>) -> Result<(), RelayError> {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            h.join().map_err(|_| RelayError::ShutdownFailed("Thread join failed".into()))?;
        }
        Ok(())
    }
}

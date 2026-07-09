use crate::network::LanDiscoveryPacket;
use bevy::prelude::Resource;
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Resource)]
pub struct LanDiscoveryListener {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    discovered: Arc<Mutex<Vec<LanDiscoveryPacket>>>,
}

impl LanDiscoveryListener {
    pub fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let discovered = Arc::new(Mutex::new(Vec::<LanDiscoveryPacket>::new()));
        let discovered_clone = discovered.clone();

        let handle = thread::spawn(move || {
            let socket = match UdpSocket::bind("0.0.0.0:9876") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[LAN] bind failed: {}", e);
                    return;
                }
            };
            let _ = socket.set_read_timeout(Some(Duration::from_millis(200)));
            let mut buf = [0u8; 64];
            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok((len, _)) = socket.recv_from(&mut buf) {
                    if let Some(pkt) = LanDiscoveryPacket::decode(&buf[..len]) {
                        let mut d = discovered_clone.lock().unwrap();
                        let pos = d.iter().position(|p| p.relay_port == pkt.relay_port);
                        if let Some(i) = pos {
                            d[i] = pkt;
                        } else {
                            d.push(pkt);
                        }
                    }
                }
            }
        });

        Self { stop, handle: Some(handle), discovered }
    }

    pub fn drain(&self) -> Vec<LanDiscoveryPacket> {
        let mut d = self.discovered.lock().unwrap();
        d.drain(..).collect()
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

impl Default for LanDiscoveryListener {
    fn default() -> Self {
        Self::start()
    }
}

impl Drop for LanDiscoveryListener {
    fn drop(&mut self) {
        self.stop();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

//! LAN discovery listener — receives UDP beacon packets.
//!
//! Listens on UDP port 9876 (shared with relay), collects `RoomAdvertisement`s
//! from all beacon broadcasters on the LAN. Deduplicates by `relay_id`.

use crate::discovery::LanDiscoveryPacket;
use bevy::prelude::Resource;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 发现层的绑定范围。
///
/// - `AllInterfaces`（默认，生产）：绑定通配地址，才能收到来自任意网卡的
///   局域网广播 beacon。
/// - `Loopback`（测试 / 本机调试）：只绑回环。生产 beacon 除广播之外**本来就会
///   向 `127.0.0.1:9876` 发一份**（见 session_host::thread 的 beacon 循环），
///   因此回环绑定足以完成本机发现；这样测试进程里不再出现通配绑定，
///   macOS 应用防火墙（ALF）也就不会反复询问"是否允许接受传入网络连接"
///   ——测试二进制每次重编哈希都变，否则该弹窗会一遍遍出现。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DiscoveryScope {
    /// 监听所有网卡（生产默认）。
    #[default]
    AllInterfaces,
    /// 仅回环（测试与本机调试）。
    Loopback,
}

/// 进程级范围开关（`AtomicU8` 而非 `OnceLock`，便于测试重复设置而不 panic）。
static DISCOVERY_SCOPE: AtomicU8 = AtomicU8::new(DiscoveryScope::AllInterfaces as u8);

/// 设置发现层绑定范围（进程级；测试与单机调试用）。
pub fn set_discovery_scope(scope: DiscoveryScope) {
    DISCOVERY_SCOPE.store(scope as u8, Ordering::Relaxed);
}

/// **仅供集成测试**：让本进程的发现/中继绑定全部走回环。
///
/// 为什么需要显式调用：测试进程里若出现通配绑定，操作系统防火墙（macOS ALF）
/// 会询问"是否允许接受传入网络连接"，而测试二进制每次重新编译哈希都会变，
/// 于是该弹窗一遍遍出现。（曾尝试用 cargo 的 `CARGO_TARGET_TMPDIR` 自动判定，
/// 实测 cargo 1.95 在集成测试中并不设置该变量，故改为显式调用。）
///
/// 生产代码不得调用：真实局域网联机依赖通配绑定收发广播。
pub fn use_loopback_bindings_for_tests() {
    set_discovery_scope(DiscoveryScope::Loopback);
}

/// 读取当前发现层绑定范围。
pub fn discovery_scope() -> DiscoveryScope {
    match DISCOVERY_SCOPE.load(Ordering::Relaxed) {
        1 => DiscoveryScope::Loopback,
        _ => DiscoveryScope::AllInterfaces,
    }
}

/// 当前范围对应的绑定地址。
pub fn discovery_bind_addr() -> IpAddr {
    match discovery_scope() {
        DiscoveryScope::Loopback => IpAddr::V4(Ipv4Addr::LOCALHOST),
        DiscoveryScope::AllInterfaces => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
    }
}

/// Listens for UDP discovery beacons and collects `RoomAdvertisement`s.
#[derive(Resource)]
pub struct LanDiscoveryListener {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    discovered: Arc<Mutex<Vec<LanDiscoveryPacket>>>,
}

impl LanDiscoveryListener {
    /// Start listening on the default discovery port (9876).
    pub fn start() -> Self {
        Self::start_on(9876)
    }

    /// Start listening on a specific UDP port, binding per [`discovery_scope`]
    /// （生产绑所有网卡；测试可先设为回环）。
    pub fn start_on(port: u16) -> Self {
        Self::start_on_addr(discovery_bind_addr(), port)
    }

    /// Start listening on an explicit bind address.
    ///
    /// 生产用通配地址（要收广播）；测试传 `127.0.0.1`，
    /// 以免触发操作系统防火墙的入站授权询问。
    pub fn start_on_addr(bind: IpAddr, port: u16) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let discovered = Arc::new(Mutex::new(Vec::<LanDiscoveryPacket>::new()));
        let discovered_clone = discovered.clone();

        let handle = thread::spawn(move || {
            let socket = match UdpSocket::bind((bind, port)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[LAN] bind failed: {}", e);
                    return;
                }
            };
            let _ = socket.set_read_timeout(Some(Duration::from_millis(200)));
            // Larger buffer for variable-length packets (room_name string, etc.)
            let mut buf = [0u8; 512];
            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok((len, _)) = socket.recv_from(&mut buf) {
                    if let Some(pkt) = LanDiscoveryPacket::decode(&buf[..len]) {
                        let mut d = discovered_clone.lock().unwrap();
                        // Deduplicate by relay_id: same relay → update, different → insert
                        let rid = pkt.advertisement.relay_id;
                        let pos = d.iter().position(|p| p.advertisement.relay_id == rid);
                        if let Some(i) = pos {
                            d[i] = pkt;
                        } else {
                            d.push(pkt);
                        }
                    }
                }
            }
        });

        Self {
            stop,
            handle: Some(handle),
            discovered,
        }
    }

    /// Drain all received packets since last call.
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

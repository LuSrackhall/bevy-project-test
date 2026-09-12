//! Relay library — thin wrapper around `bevy_adapter::relay_core` for CLI usage.
//!
//! Provides `start_relay()` that creates a TCP listener and delegates to the
//! shared relay runtime.

use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;

use tokio::net::UdpSocket;

use bevy_adapter::discovery::RelayId;
use bevy_adapter::relay_core::{self, RelayConfig};

// 供集成测试切换发现/中继的绑定范围（回环可避免 OS 防火墙的入站授权询问）。
pub use bevy_adapter::lan::{discovery_scope, set_discovery_scope, DiscoveryScope};

/// Start the relay server. Accepts connections until shutdown.
///
/// If `relay_id` is `None`, a random `RelayId` is generated.
pub async fn start_relay(
    port: u16,
    seed: u64,
    player_count: u8,
    relay_id: Option<RelayId>,
) -> Result<(), Box<dyn std::error::Error>> {
    let relay_id = relay_id.unwrap_or_else(|| RelayId(rand::random::<u64>()));

    // 生产：显式双栈（Windows 的 IPV6_V6ONLY 默认为 1，必须显式关闭才能收 IPv4 客户端）。
    // 测试范围：IPv4 回环。
    let socket = match discovery_scope() {
        DiscoveryScope::Loopback => UdpSocket::bind(("127.0.0.1", port)).await?,
        DiscoveryScope::AllInterfaces => bevy_adapter::transport::bind_dual_stack_udp(port)?,
    };
    println!(
        "Relay on port {} (players={}, seed={})",
        port, player_count, seed
    );

    let config = RelayConfig {
        relay_id,
        game_id: 1,
        ruleset_version: 1,
        seed,
        map_spec_hash: 0,
        map_size: simulation::map::MapSize::Medium,
        player_count,
        input_delay: 3,
        current_clients: Arc::new(AtomicUsize::new(0)),
    };
    let stop = AtomicBool::new(false);
    relay_core::run_relay(socket, config, &stop).await;

    Ok(())
}

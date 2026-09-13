//! Relay server binary — starts the TCP relay.
//!
//! Usage: relay --port <port> --seed <seed> --players <count> [--relay-id <id>] [--loopback]
//!             [--latency-ms <ms>] [--jitter-ms <ms>] [--loss-pct <pct>]
//!
//! `--relay-id` 固定 relay 身份：脚本化本地联机必须显式给，否则每次启动都是随机 id，
//! 客户端会以 `Join rejected: Relay identity mismatch` 被拒（客户端侧同名的 `--relay-id`
//! 必须传同一个值）。
//!
//! `--loopback` 仅供本地/同机测试：绑定 127.0.0.1 而不是通配地址，
//! 避免操作系统防火墙弹出"是否允许接受传入网络连接"（外部机器本来也连不到测试 relay）。
//!
//! `--latency-ms/--jitter-ms/--loss-pct` 在**广播出口**注入链路条件（见
//! `bevy_adapter::netem`）：loopback 上 RTT≈0、几乎不丢包，本地稳态测不出真实
//! WiFi/公网的症状，需要显式注入才能复现"联机卡顿"。全部为 0 时不注入（生产行为）。

use relay::{
    set_discovery_scope, start_relay_with, DiscoveryScope, NetemConfig,
};

use bevy_adapter::discovery::RelayId;

fn parse_arg<T: std::str::FromStr>(args: &[String], name: &str) -> Option<T> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = parse_arg(&args, "--port").unwrap_or(9876);
    let seed: u64 = parse_arg(&args, "--seed").unwrap_or(42);
    let players: u8 = parse_arg(&args, "--players").unwrap_or(2);
    let relay_id: Option<RelayId> = parse_arg::<u64>(&args, "--relay-id").map(RelayId);
    let latency_ms: u32 = parse_arg(&args, "--latency-ms").unwrap_or(0);
    let jitter_ms: u32 = parse_arg(&args, "--jitter-ms").unwrap_or(0);
    let loss_pct: u8 = parse_arg(&args, "--loss-pct").unwrap_or(0);

    if args.iter().any(|a| a == "--loopback") {
        set_discovery_scope(DiscoveryScope::Loopback);
    }

    // 全 0 ⇒ None（生产行为，零开销）；seed 固定为 42，使注入序列可复现。
    let netem_cfg = NetemConfig::new(latency_ms, jitter_ms, loss_pct, 42);
    let netem = if netem_cfg.is_noop() {
        None
    } else {
        eprintln!(
            "[RELAY] netem 注入: {}ms ±{}ms, loss {}%",
            netem_cfg.latency_ms, netem_cfg.jitter_ms, netem_cfg.loss_pct
        );
        Some(netem_cfg)
    };

    start_relay_with(port, seed, players, relay_id, netem).await
}

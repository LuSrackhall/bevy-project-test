//! Relay server binary — starts the TCP relay.
//!
//! Usage: relay --port <port> --seed <seed> --players <count> [--relay-id <id>] [--loopback]
//!
//! `--relay-id` 固定 relay 身份：脚本化本地联机必须显式给，否则每次启动都是随机 id，
//! 客户端会以 `Join rejected: Relay identity mismatch` 被拒（客户端侧同名的 `--relay-id`
//! 必须传同一个值）。
//!
//! `--loopback` 仅供本地/同机测试：绑定 127.0.0.1 而不是通配地址，
//! 避免操作系统防火墙弹出"是否允许接受传入网络连接"（外部机器本来也连不到测试 relay）。

use relay::{set_discovery_scope, start_relay, DiscoveryScope};

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

    if args.iter().any(|a| a == "--loopback") {
        set_discovery_scope(DiscoveryScope::Loopback);
    }

    start_relay(port, seed, players, relay_id).await
}

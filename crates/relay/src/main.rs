//! Relay server binary — starts the TCP relay.
//!
//! Usage: relay --port <port> --seed <seed> --players <count>

use relay::start_relay;

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

    start_relay(port, seed, players, None).await
}

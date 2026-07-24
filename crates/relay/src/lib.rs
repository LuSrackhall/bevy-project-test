//! Relay library — thin wrapper around `bevy_adapter::relay_core` for CLI usage.
//!
//! Provides `start_relay()` that creates a TCP listener and delegates to the
//! shared relay runtime.

use std::sync::atomic::AtomicBool;

use tokio::net::TcpListener;

use bevy_adapter::discovery::RelayId;
use bevy_adapter::relay_core::{self, RelayConfig};

/// Start the relay server. Accepts connections until shutdown.
pub async fn start_relay(
    port: u16,
    seed: u64,
    player_count: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let relay_id = RelayId(rand::random::<u64>());

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Relay on port {} (players={}, seed={})", port, player_count, seed);

    let config = RelayConfig {
        relay_id,
        game_id: 1,
        ruleset_version: 1,
        seed,
        map_spec_hash: 0,
        player_count,
        input_delay: 3,
    };
    let stop = AtomicBool::new(false);
    relay_core::run_relay(listener, config, &stop).await;

    Ok(())
}

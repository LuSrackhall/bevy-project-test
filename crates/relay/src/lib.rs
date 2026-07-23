//! Relay library — thin wrapper around `bevy_adapter::relay_core` for CLI usage.
//!
//! Provides `start_relay()` that creates a TCP listener and delegates to the
//! shared relay runtime.
//!
//! TODO (Change B2): Remove the redundant UDP beacon — ThreadRelayRuntime's
//! beacon (Change A) is the canonical version.

use std::sync::atomic::AtomicBool;

use tokio::net::{TcpListener, UdpSocket};
use tokio::time::Duration;

use bevy_adapter::discovery::{
    LanDiscoveryPacket, RelayId, RoomAdvertisement, RoomId, RoomMetadata, RoomState,
};
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

    // Spawn UDP broadcast for LAN discovery
    // TODO (Change B2): Remove this redundant beacon — ThreadRelayRuntime's
    // beacon (Change A) is the canonical version with RoomMetadata.
    let udp_socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
    udp_socket.set_broadcast(true)?;
    let bc_port = port;
    let bc_relay_id = relay_id;
    let room_id = RoomId(rand::random::<u64>());
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let packet = LanDiscoveryPacket::new(RoomAdvertisement {
                relay_id: bc_relay_id,
                endpoint: format!("127.0.0.1:{}", bc_port),
                room: RoomMetadata {
                    room_id,
                    room_name: String::new(),
                    map_id: "grassland_small".into(),
                    current_players: 0,
                    max_players: player_count,
                    state: RoomState::Waiting,
                },
            });
            if let Ok(data) = packet.encode() {
                let _ = udp_socket.send_to(&data, "255.255.255.255:9876").await;
            }
        }
    });

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

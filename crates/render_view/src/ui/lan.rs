use crate::GameState;
use bevy::prelude::*;
use bevy_adapter::lan::LanDiscoveryListener;
use bevy_adapter::network::LanDiscoveryPacket;
use std::time::Instant;

const LAN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(Resource, Default)]
pub struct LanServers {
    pub servers: Vec<LanServerEntry>,
}

#[derive(Clone)]
pub struct LanServerEntry {
    pub packet: LanDiscoveryPacket,
    pub last_seen: Instant,
}

pub fn start_lan_discovery(mut commands: Commands) {
    commands.init_resource::<LanServers>();
    commands.init_resource::<LanDiscoveryListener>();
}

pub fn stop_lan_discovery(mut commands: Commands) {
    commands.remove_resource::<LanDiscoveryListener>();
    commands.remove_resource::<LanServers>();
}

pub fn update_lan_servers(
    mut servers: ResMut<LanServers>,
    listener: Option<Res<LanDiscoveryListener>>,
) {
    // Timeout expired entries
    servers.servers.retain(|s| s.last_seen.elapsed() < LAN_TIMEOUT);

    // Drain new discoveries
    if let Some(listener) = listener {
        let new_packets = listener.drain();
        for pkt in new_packets {
            let pos = servers.servers.iter().position(|s| s.packet.relay_port == pkt.relay_port);
            if let Some(i) = pos {
                servers.servers[i].packet = pkt;
                servers.servers[i].last_seen = Instant::now();
            } else {
                servers.servers.push(LanServerEntry {
                    packet: pkt,
                    last_seen: Instant::now(),
                });
            }
        }
    }
}

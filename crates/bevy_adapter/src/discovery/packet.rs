//! LAN Discovery UDP packet — transport envelope for `RoomAdvertisement`.
//!
//! Replaces the old fixed-size manual-encoded `LanDiscoveryPacket` in `network.rs`.
//! Uses bincode serialization (already a project dependency) for variable-length fields.

use serde::{Deserialize, Serialize};

use super::model::RoomAdvertisement;

/// Protocol magic constant for validating incoming UDP beacons.
pub const DISCOVERY_MAGIC: [u8; 2] = [b'R', b'T'];
/// Current discovery protocol version.
pub const DISCOVERY_VERSION: u16 = 1;

/// UDP beacon packet for LAN room discovery.
///
/// Transport envelope containing:
/// - `magic` / `version`: protocol identification
/// - `advertisement`: the actual room discovery payload
///
/// Serialized with bincode for variable-length field support
/// (room_name string, map_id string, etc.).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LanDiscoveryPacket {
    pub magic: [u8; 2],
    pub version: u16,
    pub advertisement: RoomAdvertisement,
}

impl LanDiscoveryPacket {
    pub fn new(advertisement: RoomAdvertisement) -> Self {
        Self {
            magic: DISCOVERY_MAGIC,
            version: DISCOVERY_VERSION,
            advertisement,
        }
    }

    /// Encode to byte buffer using bincode.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        bincode::serde::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| format!("Discovery encode failed: {}", e))
    }

    /// Decode from byte buffer using bincode.
    pub fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() < 4 {
            return None;
        }
        // Validate magic before full decode (quick rejection of non-discovery packets)
        if buf[0..2] != DISCOVERY_MAGIC {
            return None;
        }
        let (pkt, _): (Self, _) = bincode::serde::decode_from_slice(buf, bincode::config::standard()).ok()?;
        Some(pkt)
    }
}

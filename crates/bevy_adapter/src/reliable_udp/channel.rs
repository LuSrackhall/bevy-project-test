//! Datagram channel abstraction — real UDP socket or test netem fake channel.
//!
//! The reliable layer operates on `DatagramChannel` so fault injection
//! (drop/reorder/duplicate/fragment) can be scripted deterministically in tests.

use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;

/// A raw datagram transport. Real impl: `tokio::net::UdpSocket`.
/// Test impl: `channel_netem::NetemChannel` (scripted fault injection + virtual clock).
#[async_trait]
pub trait DatagramChannel: Send {
    /// Send a datagram to `to`.
    async fn send_to(&mut self, buf: &[u8], to: SocketAddr) -> io::Result<()>;
    /// Receive a datagram into `buf`, returning (bytes_read, source_addr).
    async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
}

//! Real UDP `DatagramChannel` implementation over `tokio::net::UdpSocket`.

use std::io;
use std::net::SocketAddr;

use async_trait::async_trait;
use tokio::net::UdpSocket;

use super::channel::DatagramChannel;

pub struct UdpChannel {
    socket: UdpSocket,
}

impl UdpChannel {
    /// Bind a dual-stack socket (IPv4/IPv6) on `addr` (e.g. `[::]:port`).
    pub async fn bind(addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self { socket })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }
}

#[async_trait]
impl DatagramChannel for UdpChannel {
    async fn send_to(&mut self, buf: &[u8], to: SocketAddr) -> io::Result<()> {
        self.socket.send_to(buf, to).await.map(|_| ())
    }

    async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf).await
    }
}

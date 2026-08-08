//! In-memory fake `DatagramChannel` for deterministic fault-injection tests.
//!
//! Unlike a real loopback UDP socket (which never drops/reorders on loopback),
//! this channel records every `send_to` into `sent` and lets the test decide
//! what to deliver to the peer — dropping, reordering, duplicating, or fragmenting.
//! It also carries a virtual clock so RTO/retransmission logic can be driven
//! deterministically with zero sleeps.

use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;

use super::channel::DatagramChannel;

pub struct NetemChannel {
    inbound: VecDeque<(SocketAddr, Vec<u8>)>,
    /// Every datagram this endpoint sent (test inspects/reinjects these).
    sent: Vec<Vec<u8>>,
    /// Virtual clock (advance manually in tests).
    now: Duration,
}

impl NetemChannel {
    pub fn new() -> Self {
        Self {
            inbound: VecDeque::new(),
            sent: Vec::new(),
            now: Duration::ZERO,
        }
    }

    /// All datagrams sent by this endpoint so far.
    pub fn sent(&self) -> &[Vec<u8>] {
        &self.sent
    }

    /// Inject an inbound datagram as if from `from` (test drives the peer).
    pub fn inject(&mut self, from: SocketAddr, data: Vec<u8>) {
        self.inbound.push_back((from, data));
    }

    /// Advance the virtual clock.
    pub fn advance(&mut self, d: Duration) {
        self.now += d;
    }

    pub fn now(&self) -> Duration {
        self.now
    }

    /// Count of sent datagrams.
    pub fn sent_count(&self) -> usize {
        self.sent.len()
    }

    /// Synchronous record of a sent datagram (avoids holding a MutexGuard across await).
    pub fn push_sent(&mut self, data: Vec<u8>) {
        self.sent.push(data);
    }

    /// Synchronous pop of one inbound datagram into `buf`.
    pub fn pop_inbound(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        if let Some((from, data)) = self.inbound.pop_front() {
            let n = data.len().min(buf.len());
            buf[..n].copy_from_slice(&data[..n]);
            Ok((n, from))
        } else {
            Err(io::Error::new(io::ErrorKind::WouldBlock, "netem inbound empty"))
        }
    }
}

#[async_trait]
impl DatagramChannel for NetemChannel {
    async fn send_to(&mut self, buf: &[u8], _to: SocketAddr) -> io::Result<()> {
        // Record only; the test decides delivery (drop/reorder/duplicate) by
        // pulling from `sent` and re-injecting into the peer's inbound.
        self.push_sent(buf.to_vec());
        Ok(())
    }

    async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.pop_inbound(buf)
    }
}

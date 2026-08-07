//! Self-written reliable UDP transport.
//!
//! Event-driven: no internal sleeps. The caller drives time via `set_now`
//! (virtual clock in tests, wall clock in production) and pumps `poll()` /
//! `process()` each iteration. Three channels: Tick (reliable ordered),
//! Control (reliable ordered), Heartbeat (unreliable). Fragmentation over MTU.
//!
//! Sending is two-phase: sync methods (`send_reliable`/`process`) stage frames
//! into `outbound`; `poll()` (async) flushes them over the `DatagramChannel`.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;

pub mod channel;
pub mod channel_netem;
pub mod channel_udp;
pub mod protocol;

use channel::DatagramChannel;
use protocol::{
    ack_frame, decode, encode, Frame, CH_HEARTBEAT, KIND_ACK, KIND_DATA, KIND_FRAG, MAX_PAYLOAD,
};

/// Max message payload that fits one datagram (MAX_PAYLOAD minus frag sub-header).
const MAX_DATA: usize = MAX_PAYLOAD - 8;

#[derive(Clone, Debug)]
pub struct ReliableConfig {
    /// Sliding window per reliable channel (max in-flight unacked).
    pub window: u32,
    /// Initial retransmission timeout.
    pub rto_initial: Duration,
    /// Max retransmissions before giving up (caller then reconnects/catches up).
    pub max_retries: u32,
    /// Min spacing between sent datagrams (pacing; no congestion control).
    pub pacing: Duration,
}

impl Default for ReliableConfig {
    fn default() -> Self {
        Self {
            window: 32,
            rto_initial: Duration::from_millis(200),
            max_retries: 5,
            pacing: Duration::from_millis(1),
        }
    }
}

#[derive(Default)]
struct ChannelSender {
    next_seq: u32,
    last_acked: u32,
    /// (seq, frame_bytes, retries, next_rto_at)
    unacked: VecDeque<(u32, Vec<u8>, u32, Duration)>,
}

#[derive(Default)]
struct ChannelReceiver {
    next_expected: u32,
    buffer: BTreeMap<u32, Vec<u8>>,
}

struct FragAssembler {
    total: u16,
    /// frag_idx → (seq, data)
    parts: BTreeMap<u16, (u32, Vec<u8>)>,
}

/// A reliable socket bound to one peer (client↔relay).
pub struct ReliableSocket {
    channel: Box<dyn DatagramChannel>,
    peer: SocketAddr,
    now: Duration,
    config: ReliableConfig,
    senders: [ChannelSender; 3],
    receivers: [ChannelReceiver; 3],
    assemblers: HashMap<(u8, u32), FragAssembler>,
    outbox: VecDeque<Vec<u8>>,
    outbound: VecDeque<Vec<u8>>,
    dead: bool,
}

impl ReliableSocket {
    pub fn new(channel: Box<dyn DatagramChannel>, peer: SocketAddr, config: ReliableConfig) -> Self {
        Self {
            channel,
            peer,
            now: Duration::ZERO,
            config,
            senders: Default::default(),
            receivers: Default::default(),
            assemblers: HashMap::new(),
            outbox: VecDeque::new(),
            outbound: VecDeque::new(),
            dead: false,
        }
    }

    /// Drive the virtual clock (tests) or wall-clock elapsed (production).
    pub fn set_now(&mut self, now: Duration) {
        self.now = now;
    }

    pub fn peer(&self) -> SocketAddr {
        self.peer
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }

    pub fn mark_dead(&mut self) {
        self.dead = true;
    }

    /// Send a reliable ordered message on `channel` (Tick or Control).
    /// Splits into fragments if over MTU. Queued for send on next poll().
    pub fn send_reliable(&mut self, channel: u8, payload: Vec<u8>) {
        let msg_id = (self.now.as_millis() as u32) ^ (channel as u32) << 24;
        if payload.len() <= MAX_DATA {
            self.queue_fragment(channel, msg_id, 0, 1, payload);
        } else {
            let total = (payload.len() + MAX_DATA - 1) / MAX_DATA;
            for idx in 0..total as u16 {
                let start = idx as usize * MAX_DATA;
                let end = (start + MAX_DATA).min(payload.len());
                self.queue_fragment(channel, msg_id, idx, total as u16, payload[start..end].to_vec());
            }
        }
    }

    /// Send an unreliable datagram (e.g. heartbeat) — no seq tracking, no retransmit.
    pub fn send_unreliable(&mut self, payload: Vec<u8>) {
        let frame = encode(CH_HEARTBEAT, 0, KIND_DATA, None, &payload);
        self.outbound.push_back(frame);
    }

    fn queue_fragment(&mut self, channel: u8, msg_id: u32, idx: u16, total: u16, data: Vec<u8>) {
        let sender = &mut self.senders[channel as usize];
        let seq = sender.next_seq;
        sender.next_seq = sender.next_seq.wrapping_add(1);
        let frag = if total > 1 { Some((msg_id, idx, total)) } else { None };
        let frame = encode(channel, seq, KIND_FRAG, frag, &data);
        sender.unacked.push_back((seq, frame, 0, self.now + self.config.rto_initial));
    }

    /// Select frames that are within the window and due (fresh or RTO exceeded)
    /// and stage them for send. Must be called each iteration before poll().
    pub fn process(&mut self) -> usize {
        let mut staged = 0;
        for ch in 0..3usize {
            let window = self.config.window;
            let max_retries = self.config.max_retries;
            let now = self.now;
            let mut new_unacked: VecDeque<(u32, Vec<u8>, u32, Duration)> = VecDeque::new();
            let sender = &mut self.senders[ch];
            for (seq, frame, retries, rto_at) in sender.unacked.drain(..) {
                let within_window = seq.wrapping_sub(sender.last_acked) <= window;
                let due = retries == 0 || now >= rto_at;
                if within_window && due {
                    if retries >= max_retries {
                        // give up: drop (caller catches up via reconnect)
                        continue;
                    }
                    self.outbound.push_back(frame.clone());
                    new_unacked.push_back((seq, frame, retries + 1, now + self.config.rto_initial));
                    staged += 1;
                } else {
                    new_unacked.push_back((seq, frame, retries, rto_at));
                }
            }
            sender.unacked = new_unacked;
        }
        staged
    }

    /// Flush outbound (async send) and pump up to N inbound datagrams with a
    /// short timeout, then return so the caller controls the loop cadence.
    pub async fn poll(&mut self) -> io::Result<()> {
        while let Some(frame) = self.outbound.pop_front() {
            self.channel.send_to(&frame, self.peer).await?;
        }
        let mut buf = [0u8; 65535];
        for _ in 0..10 {
            let res =
                tokio::time::timeout(Duration::from_millis(10), self.channel.recv_from(&mut buf)).await;
            match res {
                Ok(Ok((n, _from))) => {
                    if let Some(frame) = decode(&buf[..n]) {
                        self.handle_frame(frame);
                    }
                }
                Ok(Err(e)) if e.kind() == io::ErrorKind::WouldBlock => break,
                Ok(Err(e)) => return Err(e),
                Err(_elapsed) => break, // timeout → return control to caller
            }
        }
        Ok(())
    }

    fn handle_frame(&mut self, frame: Frame) {
        match frame.kind {
            KIND_ACK => {
                let ch = frame.channel as usize;
                if ch < 3 && frame.payload.len() == 4 {
                    let acked = u32::from_be_bytes([frame.payload[0], frame.payload[1], frame.payload[2], frame.payload[3]]);
                    let sender = &mut self.senders[ch];
                    if acked.wrapping_sub(sender.last_acked) > 0 {
                        sender.last_acked = acked;
                        // keep only frames with seq strictly after the cumulative ack
                        sender.unacked.retain(|(seq, _, _, _)| {
                            let d = seq.wrapping_sub(acked);
                            d > 0 && d < (1u32 << 31)
                        });
                    }
                }
            }
            KIND_DATA | KIND_FRAG => self.receive_data(frame),
            _ => {}
        }
    }

    fn receive_data(&mut self, frame: Frame) {
        let ch = frame.channel as usize;
        if ch >= 3 {
            return;
        }
        // Acknowledge every received reliable frame (cumulative).
        let ack = ack_frame(frame.channel, frame.seq);
        self.outbound.push_back(ack);

        match frame.frag {
            None => self.stage_data(ch, frame.seq, frame.payload),
            Some((msg_id, idx, total)) => {
                let assembler = self.assemblers.entry((frame.channel, msg_id)).or_insert(FragAssembler {
                    total,
                    parts: BTreeMap::new(),
                });
                assembler.parts.insert(idx, (frame.seq, frame.payload));
                if assembler.parts.len() as u16 == assembler.total {
                    let mut data = Vec::new();
                    let mut max_seq = 0u32;
                    for (_, (seq, part)) in &assembler.parts {
                        data.extend_from_slice(part);
                        max_seq = max_seq.max(*seq);
                    }
                    self.assemblers.remove(&(frame.channel, msg_id));
                    // Reassembled message spans multiple frame seqs: advance the
                    // expected counter past all of them and deliver directly.
                    let recv = &mut self.receivers[ch];
                    recv.next_expected = recv.next_expected.max(max_seq.wrapping_add(1));
                    self.outbox.push_back(data);
                }
            }
        }
    }

    fn stage_data(&mut self, ch: usize, seq: u32, payload: Vec<u8>) {
        let recv = &mut self.receivers[ch];
        let d = seq.wrapping_sub(recv.next_expected);
        if d == 0 {
            // in-order: deliver and advance, then drain contiguous buffered
            recv.next_expected = recv.next_expected.wrapping_add(1);
            self.outbox.push_back(payload);
            while let Some(x) = recv.buffer.remove(&recv.next_expected) {
                recv.next_expected = recv.next_expected.wrapping_add(1);
                self.outbox.push_back(x);
            }
        } else if d < (1u32 << 31) {
            // future seq (within the forward window): buffer
            recv.buffer.entry(seq).or_insert(payload);
        } else {
            // stale / duplicate (seq already passed): drop
        }
    }

    /// Take messages delivered in order.
    pub fn take_messages(&mut self) -> Vec<Vec<u8>> {
        self.outbox.drain(..).collect()
    }

    /// Take only messages matching `pred`; non-matching messages stay in the
    /// outbox (not lost) so later calls can consume them.
    pub fn take_messages_matching<F: Fn(&[u8]) -> bool>(&mut self, pred: F) -> Vec<Vec<u8>> {
        let mut kept = VecDeque::new();
        let mut taken = Vec::new();
        while let Some(m) = self.outbox.pop_front() {
            if pred(&m) {
                taken.push(m);
            } else {
                kept.push_back(m);
            }
        }
        self.outbox = kept;
        taken
    }

    /// Number of in-flight unacked frames across reliable channels.
    pub fn in_flight(&self) -> usize {
        self.senders.iter().map(|s| s.unacked.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reliable_udp::channel_netem::NetemChannel;
    use crate::reliable_udp::protocol::{CH_CONTROL, CH_TICK};
    use async_trait::async_trait;

    /// Shared netem channel so tests can inspect `sent` and inject inbound.
    struct SharedNetem(std::sync::Arc<std::sync::Mutex<NetemChannel>>);

    #[async_trait]
    impl DatagramChannel for SharedNetem {
        async fn send_to(&mut self, buf: &[u8], _to: SocketAddr) -> io::Result<()> {
            self.0.lock().unwrap().push_sent(buf.to_vec());
            Ok(())
        }
        async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
            self.0.lock().unwrap().pop_inbound(buf)
        }
    }

    fn mk_socket() -> (ReliableSocket, std::sync::Arc<std::sync::Mutex<NetemChannel>>) {
        let ch = std::sync::Arc::new(std::sync::Mutex::new(NetemChannel::new()));
        let sock = ReliableSocket::new(
            Box::new(SharedNetem(ch.clone())),
            "127.0.0.1:9000".parse().unwrap(),
            ReliableConfig::default(),
        );
        (sock, ch)
    }

    /// Drive one poll round on a socket (flush outbound + process inbound).
    fn pump(sock: &mut ReliableSocket) {
        let rt = tokio::runtime::Builder::new_current_thread().enable_io().enable_time().build().unwrap();
        rt.block_on(async {
            sock.process();
            sock.poll().await.unwrap();
        });
    }

    /// Move every datagram `a` sent into `b`'s inbound (full reliable delivery).
    fn deliver_all(a_ch: &std::sync::Arc<std::sync::Mutex<NetemChannel>>, b_ch: &std::sync::Arc<std::sync::Mutex<NetemChannel>>) {
        let sent: Vec<Vec<u8>> = a_ch.lock().unwrap().sent().to_vec();
        for d in sent {
            b_ch.lock().unwrap().inject("127.0.0.1:9001".parse().unwrap(), d);
        }
    }

    #[test]
    fn test_in_order_delivery() {
        let (mut a, ch_a) = mk_socket();
        let (mut b, ch_b) = mk_socket();

        a.send_reliable(CH_TICK, b"m1".to_vec());
        a.send_reliable(CH_TICK, b"m2".to_vec());
        pump(&mut a);
        deliver_all(&ch_a, &ch_b);
        pump(&mut b);

        let msgs = b.take_messages();
        assert_eq!(msgs, vec![b"m1".to_vec(), b"m2".to_vec()], "ordered delivery");
    }

    #[test]
    fn test_out_of_order_reordering() {
        let (mut a, ch_a) = mk_socket();
        let (mut b, ch_b) = mk_socket();

        a.send_reliable(CH_TICK, b"m1".to_vec());
        a.send_reliable(CH_TICK, b"m2".to_vec());
        a.send_reliable(CH_TICK, b"m3".to_vec());
        pump(&mut a);
        let sent = ch_a.lock().unwrap().sent().to_vec();
        assert_eq!(sent.len(), 3);
        // deliver in reverse order to force reordering
        for i in (0..3).rev() {
            ch_b.lock().unwrap().inject("127.0.0.1:9001".parse().unwrap(), sent[i].clone());
        }
        pump(&mut b);

        let msgs = b.take_messages();
        assert_eq!(msgs, vec![b"m1".to_vec(), b"m2".to_vec(), b"m3".to_vec()], "reordered back to seq order");
    }

    #[test]
    fn test_duplicate_dropped() {
        let (mut a, ch_a) = mk_socket();
        let (mut b, ch_b) = mk_socket();

        a.send_reliable(CH_TICK, b"m".to_vec());
        pump(&mut a);
        let sent = ch_a.lock().unwrap().sent().to_vec();
        assert_eq!(sent.len(), 1);
        // deliver twice (duplicate)
        ch_b.lock().unwrap().inject("127.0.0.1:9001".parse().unwrap(), sent[0].clone());
        ch_b.lock().unwrap().inject("127.0.0.1:9001".parse().unwrap(), sent[0].clone());
        pump(&mut b);

        let msgs = b.take_messages();
        assert_eq!(msgs.len(), 1, "duplicate must not deliver twice");
    }

    #[test]
    fn test_retransmission_after_rto() {
        let (mut a, ch_a) = mk_socket();
        let (mut b, ch_b) = mk_socket();

        a.send_reliable(CH_TICK, b"cmd".to_vec());
        pump(&mut a);
        // drop A's first datagram entirely (never delivered to B)
        let sent_count = ch_a.lock().unwrap().sent_count();
        assert_eq!(sent_count, 1);
        // advance virtual clock past RTO and reprocess → retransmit
        a.set_now(Duration::from_millis(250));
        pump(&mut a);
        let sent_count2 = ch_a.lock().unwrap().sent_count();
        assert!(sent_count2 >= 2, "RTO must trigger retransmission, got {}", sent_count2);

        // deliver all A sent (including retransmit) to B
        deliver_all(&ch_a, &ch_b);
        pump(&mut b);
        let msgs = b.take_messages();
        assert_eq!(msgs, vec![b"cmd".to_vec()], "retransmitted frame delivered once");
    }

    #[test]
    fn test_fragmentation_reassembly() {
        let (mut a, ch_a) = mk_socket();
        let (mut b, ch_b) = mk_socket();

        // payload larger than MAX_DATA forces fragmentation
        let big = vec![0xABu8; MAX_DATA * 2 + 10];
        a.send_reliable(CH_TICK, big.clone());
        pump(&mut a);
        let sent_count = ch_a.lock().unwrap().sent_count();
        assert!(sent_count >= 2, "large payload must fragment");

        deliver_all(&ch_a, &ch_b);
        pump(&mut b);
        let msgs = b.take_messages();
        assert_eq!(msgs.len(), 1, "fragments reassembled into one message");
        assert_eq!(msgs[0], big, "reassembled payload matches original");
    }

    #[test]
    fn test_control_channel_isolated() {
        let (mut a, ch_a) = mk_socket();
        let (mut b, ch_b) = mk_socket();

        a.send_reliable(CH_TICK, b"tick1".to_vec());
        a.send_reliable(CH_CONTROL, b"control1".to_vec());
        pump(&mut a);
        deliver_all(&ch_a, &ch_b);
        pump(&mut b);

        let msgs = b.take_messages();
        assert_eq!(msgs.len(), 2, "both channels deliver their messages");
    }
}

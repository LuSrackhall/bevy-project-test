//! Wire frame format for the reliable UDP layer.
//!
//! Frame layout (hand-rolled binary, no serde on the hot path):
//! ```text
//! [magic: u16][channel: u8][seq: u32][kind: u8][flags: u8][payload_len: u16][payload...]
//! ```
//! - `channel`: 0=Tick, 1=Control, 2=Heartbeat
//! - `seq`: monotonic per reliable channel; 0 for heartbeat (unreliable)
//! - `kind`: 0=Data, 1=ACK, 2=Frag
//! - For ACK frames payload is the cumulative acked seq (4 bytes, big-endian).
//! - For Frag frames payload starts with `[msg_id: u32][idx: u16][total: u16]` then data.

pub const MAGIC: u16 = 0x52D1;

pub const CH_TICK: u8 = 0;
pub const CH_CONTROL: u8 = 1;
pub const CH_HEARTBEAT: u8 = 2;

pub const KIND_DATA: u8 = 0;
pub const KIND_ACK: u8 = 1;
pub const KIND_FRAG: u8 = 2;

/// Max payload per datagram — safe under IPv6 minimum MTU (1280) minus headers.
pub const MAX_PAYLOAD: usize = 1200;

pub const HEADER_LEN: usize = 11; // magic(2) + channel(1) + seq(4) + kind(1) + flags(1) + len(2)

/// A parsed frame (payload excludes the wire header; for KIND_FRAG the payload
/// includes the frag sub-header).
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub channel: u8,
    pub seq: u32,
    pub kind: u8,
    /// (msg_id, frag_idx, frag_total) for fragmented data.
    pub frag: Option<(u32, u16, u16)>,
    /// Message data (for KIND_DATA), cumulative acked seq (KIND_ACK),
    /// or fragment data (KIND_FRAG, after the frag sub-header).
    pub payload: Vec<u8>,
}

pub fn encode(channel: u8, seq: u32, kind: u8, frag: Option<(u32, u16, u16)>, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len() + 8);
    out.extend_from_slice(&MAGIC.to_be_bytes());
    out.push(channel);
    out.extend_from_slice(&seq.to_be_bytes());
    out.push(kind);
    let flags: u8 = if frag.is_some() { 1 } else { 0 };
    out.push(flags);
    // payload len includes frag sub-header if present
    let extra = if frag.is_some() { 8 } else { 0 };
    let plen = payload.len() + extra;
    out.extend_from_slice(&(plen as u16).to_be_bytes());
    if let Some((msg_id, idx, total)) = frag {
        out.extend_from_slice(&msg_id.to_be_bytes());
        out.extend_from_slice(&idx.to_be_bytes());
        out.extend_from_slice(&total.to_be_bytes());
    }
    out.extend_from_slice(payload);
    out
}

/// Decode a datagram into a Frame. Returns None if the magic is wrong or the
/// datagram is malformed (len mismatch).
pub fn decode(buf: &[u8]) -> Option<Frame> {
    if buf.len() < HEADER_LEN {
        return None;
    }
    if u16::from_be_bytes([buf[0], buf[1]]) != MAGIC {
        return None;
    }
    let channel = buf[2];
    let seq = u32::from_be_bytes([buf[3], buf[4], buf[5], buf[6]]);
    let kind = buf[7];
    let flags = buf[8];
    let plen = u16::from_be_bytes([buf[9], buf[10]]) as usize;
    let has_frag = flags & 1 == 1;
    let extra = if has_frag { 8 } else { 0 };
    if buf.len() < HEADER_LEN + plen {
        return None;
    }
    let mut payload_start = HEADER_LEN;
    let payload_end = payload_start + plen;
    if payload_end > buf.len() {
        return None;
    }
    let frag = if has_frag {
        if payload_start + 8 > payload_end {
            return None;
        }
        let msg_id = u32::from_be_bytes([
            buf[payload_start],
            buf[payload_start + 1],
            buf[payload_start + 2],
            buf[payload_start + 3],
        ]);
        let idx = u16::from_be_bytes([buf[payload_start + 4], buf[payload_start + 5]]);
        let total = u16::from_be_bytes([buf[payload_start + 6], buf[payload_start + 7]]);
        payload_start += 8;
        Some((msg_id, idx, total))
    } else {
        None
    };
    let data_len = plen - extra;
    let payload = buf[payload_start..(payload_start + data_len)].to_vec();
    Some(Frame {
        channel,
        seq,
        kind,
        frag,
        payload,
    })
}

/// Build an ACK frame for a reliable channel with a cumulative acked seq.
pub fn ack_frame(channel: u8, cumulative: u32) -> Vec<u8> {
    encode(channel, cumulative, KIND_ACK, None, &cumulative.to_be_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let f = encode(CH_TICK, 42, KIND_DATA, None, b"hello");
        let d = decode(&f).unwrap();
        assert_eq!(d.channel, CH_TICK);
        assert_eq!(d.seq, 42);
        assert_eq!(d.kind, KIND_DATA);
        assert_eq!(d.frag, None);
        assert_eq!(d.payload, b"hello");
    }

    #[test]
    fn test_encode_decode_frag() {
        let f = encode(CH_CONTROL, 7, KIND_FRAG, Some((100, 0, 3)), b"frag0");
        let d = decode(&f).unwrap();
        assert_eq!(d.channel, CH_CONTROL);
        assert_eq!(d.seq, 7);
        assert_eq!(d.frag, Some((100, 0, 3)));
        assert_eq!(d.payload, b"frag0");
    }

    #[test]
    fn test_decode_bad_magic_returns_none() {
        let mut f = encode(CH_TICK, 1, KIND_DATA, None, b"x");
        f[0] = 0;
        assert!(decode(&f).is_none());
    }

    #[test]
    fn test_decode_truncated_returns_none() {
        let f = encode(CH_TICK, 1, KIND_DATA, None, b"abcdefghij");
        assert!(decode(&f[..8]).is_none());
    }
}

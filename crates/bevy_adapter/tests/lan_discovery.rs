//! LAN discovery regression tests.
//!
//! Root cause being guarded: the host's beacon bound `0.0.0.0:9876`, which
//! collided with the `LanDiscoveryListener` (also `0.0.0.0:9876`, active while
//! browsing the room list). The beacon's bind failed with EADDRINUSE and was
//! silently disabled → the host never broadcast its room → other machines on the
//! LAN could not discover it (Mac + Win symmetric).
//!
//! Fix: the beacon binds an EPHEMERAL port (`0.0.0.0:0`) and sends TO `:9876`;
//! the listener matches rooms by the packet's `relay_id`, not the source port.

use std::net::UdpSocket;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use bevy_adapter::discovery::{
    LanDiscoveryPacket, RelayId, RoomAdvertisement, RoomId, RoomMetadata, RoomState,
};
use bevy_adapter::lan::LanDiscoveryListener;
use bevy_adapter::session_host::{SessionController, ThreadRelayRuntime};

/// Tests that bind the fixed discovery port 9876 MUST hold this lock — cargo
/// runs tests in the same binary in parallel, so two 9876 binds would collide.
static PORT_9876: Mutex<()> = Mutex::new(());

fn room(room_id: u64) -> RoomMetadata {
    RoomMetadata {
        room_id: RoomId(room_id),
        room_name: format!("test-{}", room_id),
        map_id: "grassland_small".into(),
        current_players: 1,
        max_players: 4,
        state: RoomState::Waiting,
    }
}

/// Poll `drain()` until a packet with `relay_id` arrives (or deadline).
fn wait_for_packet(listener: &LanDiscoveryListener, rid: RelayId, secs: u64) -> LanDiscoveryPacket {
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        if let Some(pkt) = listener.drain().into_iter().find(|p| p.advertisement.relay_id == rid) {
            return pkt;
        }
        assert!(Instant::now() < deadline, "beacon for relay {:?} never arrived", rid);
        std::thread::sleep(Duration::from_millis(100));
    }
}

// ── R3: packet encode/decode ──────────────────────────────────────────

#[test]
fn test_packet_round_trip() {
    let ad = RoomAdvertisement {
        relay_id: RelayId(123),
        endpoint: "192.168.1.5:54321".into(),
        room: room(9),
    };
    let pkt = LanDiscoveryPacket::new(ad.clone());
    let data = pkt.encode().unwrap();
    let decoded = LanDiscoveryPacket::decode(&data).expect("decode");
    assert_eq!(decoded.magic, *b"RT");
    assert_eq!(decoded.advertisement.relay_id, ad.relay_id);
    assert_eq!(decoded.advertisement.room.room_name, "test-9");
}

#[test]
fn test_packet_rejects_garbage() {
    assert!(LanDiscoveryPacket::decode(b"not-a-packet").is_none());
    // Wrong magic but long enough to parse.
    let mut data = LanDiscoveryPacket::new(RoomAdvertisement {
        relay_id: RelayId(1),
        endpoint: "e".into(),
        room: room(1),
    })
    .encode()
    .unwrap();
    data[0] = b'X';
    assert!(LanDiscoveryPacket::decode(&data).is_none());
}

// ── S1: old beacon bind path conflicts with the browsing listener ──────

#[test]
fn test_beacon_bind_to_9876_conflicts_with_listener() {
    let _guard = PORT_9876.lock().unwrap();
    // A socket holds 0.0.0.0:9876 synchronously (simulating the browsing
    // listener, whose bind is async in a thread — avoid that race here).
    let _holder = UdpSocket::bind("0.0.0.0:9876").expect("holder binds 9876");
    // A second bind to 0.0.0.0:9876 (the OLD beacon path) must fail.
    let err = UdpSocket::bind("0.0.0.0:9876");
    assert!(
        err.is_err(),
        "old beacon bind to 9876 must conflict with the browsing listener (EADDRINUSE)"
    );
    // Sanity: an ephemeral bind (the fix) succeeds while 9876 is held.
    let fixed = UdpSocket::bind("0.0.0.0:0").expect("ephemeral beacon bind succeeds");
    assert!(fixed.set_broadcast(true).is_ok());
}

// ── R1: production beacon path reaches the listener (red → green) ──────

#[test]
fn test_beacon_reaches_listener_while_browsing() {
    let _guard = PORT_9876.lock().unwrap();
    // Browsing listener holds 9876 (the scenario that disabled the old beacon).
    let listener = LanDiscoveryListener::start_on(9876);

    // Start a real relay via the production path (ThreadRelayRuntime → beacon).
    let mut ctrl = SessionController::new(Box::new(ThreadRelayRuntime));
    ctrl.create_session(room(42)).expect("relay starts");
    let rid = ctrl.current_session().unwrap().relay.relay_id();

    // The beacon must reach the listener even though the listener holds 9876.
    let pkt = wait_for_packet(&listener, rid, 6);
    assert_eq!(pkt.advertisement.room.room_name, "test-42");

    ctrl.destroy_session().expect("relay stops");
}

// ── R2: beacon broadcasts from an ephemeral (non-9876) source port ─────

#[test]
fn test_beacon_source_port_is_not_9876() {
    let _guard = PORT_9876.lock().unwrap();
    // A raw socket on 9876 (not the production listener, so we can read the
    // source port) captures the beacon. This only works once the beacon binds
    // an ephemeral port — before the fix it failed to bind at all.
    let probe = UdpSocket::bind("0.0.0.0:9876").expect("probe binds 9876");
    probe.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

    let mut ctrl = SessionController::new(Box::new(ThreadRelayRuntime));
    ctrl.create_session(room(7)).expect("relay starts");

    let mut buf = [0u8; 512];
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        match probe.recv_from(&mut buf) {
            Ok((len, src)) => {
                if let Some(pkt) = LanDiscoveryPacket::decode(&buf[..len]) {
                    assert_eq!(pkt.advertisement.room.room_name, "test-7");
                    assert_ne!(
                        src.port(),
                        9876,
                        "beacon must broadcast from an ephemeral source port, not 9876"
                    );
                    break;
                }
            }
            Err(_) => {}
        }
        assert!(Instant::now() < deadline, "beacon never reached the 9876 probe");
    }

    ctrl.destroy_session().expect("relay stops");
}

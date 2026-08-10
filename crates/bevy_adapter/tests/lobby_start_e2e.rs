//! E2E: two clients through the REAL client transport (spawn_network_client_nonblocking
//! → udp_session → reliable UDP) must reach GameStarted via BOTH start paths:
//!   A. auto-start: both clients uplink PlayerTick frames while in the lobby
//!      (the relay's on_player_frame flips game_started when all seats connect);
//!   B. ready-path: both clients signal LobbyReady (relay broadcasts GameStarted
//!      on all-ready).
//!
//! If either path fails to deliver GameStarted, the render_view lobby can never
//! leave Lobby — the "无法开局" symptom.

use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use bevy_adapter::discovery::RelayId;
use bevy_adapter::network::{NetworkEvent, NetworkEventReceiver, PlayerTickFrame};
use bevy_adapter::transport::{
    NetworkClientHandle, NetworkSender, spawn_network_client_nonblocking,
};

fn find_free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

fn spawn_relay(port: u16, seed: u64, players: u8, relay_id: RelayId) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("relay".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .expect("relay tokio runtime");
            rt.block_on(relay::start_relay(port, seed, players, Some(relay_id)))
                .expect("relay server")
        })
        .expect("relay thread")
}

/// Drain the event receiver until `pred` matches, returning the matched event.
/// Any non-matching events in the drained batch are discarded (test-only).
fn wait_event(
    ev: &NetworkEventReceiver,
    label: &str,
    pred: impl Fn(&NetworkEvent) -> bool,
) -> NetworkEvent {
    let start = std::time::Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "timeout waiting for {label}"
        );
        for e in ev.drain_all() {
            if pred(&e) {
                return e;
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_joined(ev: &NetworkEventReceiver, label: &str) -> u8 {
    match wait_event(ev, &format!("{label} GameJoined"), |e| {
        matches!(e, NetworkEvent::GameJoined { .. })
    }) {
        NetworkEvent::GameJoined { player_id, .. } => player_id,
        _ => unreachable!(),
    }
}

/// Connect two real clients to a fresh 2-player relay. Returns the event
/// receivers, senders, and the client handles — the handles MUST be kept alive
/// (dropping them stops the client thread via `NetworkClientHandle::drop`).
#[allow(clippy::type_complexity)]
fn connect_two(
    port: u16,
) -> (
    NetworkEventReceiver,
    NetworkEventReceiver,
    NetworkSender,
    NetworkSender,
    NetworkClientHandle,
    NetworkClientHandle,
    u8,
    u8,
) {
    let ev0 = NetworkEventReceiver::default();
    let ev1 = NetworkEventReceiver::default();
    let (_r0, s0, h0, _st0) =
        spawn_network_client_nonblocking(format!("127.0.0.1:{port}"), 1, 0, 1, ev0.clone(), RelayId(42));
    let (_r1, s1, h1, _st1) =
        spawn_network_client_nonblocking(format!("127.0.0.1:{port}"), 1, 0, 1, ev1.clone(), RelayId(42));

    let pid0 = wait_joined(&ev0, "client0");
    let pid1 = wait_joined(&ev1, "client1");
    assert_ne!(pid0, pid1, "relay must assign distinct player ids");

    (ev0, ev1, s0, s1, h0, h1, pid0, pid1)
}

/// Path A: auto-start. Both clients uplink a tick-1 frame while in the lobby;
/// the relay flips game_started when the last seat connects and broadcasts GameStarted.
#[test]
fn test_auto_start_via_tick_frames() {
    let port = find_free_port();
    let _relay = spawn_relay(port, 42, 2, RelayId(42));
    thread::sleep(Duration::from_millis(200));

    let (ev0, ev1, s0, s1, _h0, _h1, pid0, pid1) = connect_two(port);

    // Mimic network_flush_system: uplink frames for tick 1 while in the lobby.
    s0.push(PlayerTickFrame {
        magic: 0xBEEF,
        version: 1,
        game_id: 1,
        tick: 1,
        player_id: pid0,
        commands: vec![],
        player_sid: 1,
    });
    s1.push(PlayerTickFrame {
        magic: 0xBEEF,
        version: 1,
        game_id: 1,
        tick: 1,
        player_id: pid1,
        commands: vec![],
        player_sid: 1,
    });

    match wait_event(&ev0, "client0 GameStarted", |e| {
        matches!(e, NetworkEvent::GameStarted { .. })
    }) {
        NetworkEvent::GameStarted { player_count, .. } => assert_eq!(player_count, 2),
        _ => unreachable!(),
    }
    match wait_event(&ev1, "client1 GameStarted", |e| {
        matches!(e, NetworkEvent::GameStarted { .. })
    }) {
        NetworkEvent::GameStarted { player_count, .. } => assert_eq!(player_count, 2),
        _ => unreachable!(),
    }
}

/// Path B: ready-path. Both clients signal LobbyReady → all-ready → GameStarted.
#[test]
fn test_ready_path_broadcasts_game_started() {
    let port = find_free_port();
    let _relay = spawn_relay(port, 42, 2, RelayId(42));
    thread::sleep(Duration::from_millis(200));

    let (ev0, ev1, s0, s1, _h0, _h1, pid0, pid1) = connect_two(port);

    // Host (player pid0) and client (player pid1) both ready.
    s0.send_lobby_ready(pid0, true);
    s1.send_lobby_ready(pid1, true);

    match wait_event(&ev0, "client0 GameStarted", |e| {
        matches!(e, NetworkEvent::GameStarted { .. })
    }) {
        NetworkEvent::GameStarted { player_count, .. } => assert_eq!(player_count, 2),
        _ => unreachable!(),
    }
    match wait_event(&ev1, "client1 GameStarted", |e| {
        matches!(e, NetworkEvent::GameStarted { .. })
    }) {
        NetworkEvent::GameStarted { player_count, .. } => assert_eq!(player_count, 2),
        _ => unreachable!(),
    }
}

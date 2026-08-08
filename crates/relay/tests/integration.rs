//! Integration test: reliable-UDP relay with simulated clients.
//!
//! Starts a relay server in-process, connects two virtual clients via the
//! reliable UDP transport, sends PlayerTickFrames, and verifies BroadcastFrames
//! arrive correctly.

use std::net::SocketAddr;
use std::time::Duration;

use bevy_adapter::discovery::{RelayId, RoomId};
use bevy_adapter::network::{
    BroadcastFrame, PlayerTickFrame, ReconnectPage, ReconnectRequest, ReconnectResponse,
    RelayClientMessage, RelayServerMessage,
};
use bevy_adapter::reliable_udp::channel_udp::UdpChannel;
use bevy_adapter::reliable_udp::protocol::{CH_CONTROL, CH_TICK};
use bevy_adapter::reliable_udp::{ReliableConfig, ReliableSocket};
use relay::start_relay;

async fn find_free_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind failed");
    listener.local_addr().unwrap().port()
}

/// Connect a UDP client, send JoinGame, and pump until GameJoined.
/// Returns the reliable socket (still owned by the caller for later pumps).
async fn udp_join(port: u16, relay_id: RelayId) -> (ReliableSocket, u8) {
    let sock = UdpChannel::bind("0.0.0.0:0").await.unwrap();
    let peer: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let mut rs = ReliableSocket::new(Box::new(sock), peer, ReliableConfig::default());
    let join = RelayClientMessage::JoinGame { room_id: RoomId(0), relay_id };
    let data = bincode::serde::encode_to_vec(&join, bincode::config::standard()).unwrap();
    rs.send_reliable(CH_CONTROL, data);

    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(5) {
            panic!("udp_join timed out waiting for GameJoined");
        }
        rs.set_now(start.elapsed());
        rs.process();
        rs.poll().await.unwrap();
        for msg in rs.take_messages() {
            if let Ok((RelayServerMessage::GameJoined { player_id, .. }, _)) =
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(&msg, bincode::config::standard())
            {
                return (rs, player_id);
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Send a PlayerTickFrame for `tick` on the tick channel.
async fn udp_send_tick(rs: &mut ReliableSocket, tick: u32, player_id: u8) {
    let frame = PlayerTickFrame {
        magic: 0xBEEF,
        version: 1,
        game_id: 1,
        tick,
        player_id,
        commands: vec![],
        player_sid: 1,
    };
    let msg = RelayClientMessage::PlayerTick(frame);
    let data = bincode::serde::encode_to_vec(&msg, bincode::config::standard()).unwrap();
    rs.send_reliable(CH_TICK, data);
    pump(rs).await;
}

/// Pump the reliable socket until a non-GameStarted relay message arrives.
async fn udp_recv_message(rs: &mut ReliableSocket, secs: u64) -> RelayServerMessage {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(secs) {
            panic!("udp_recv_message timed out");
        }
        pump(rs).await;
        for msg in rs.take_messages() {
            if let Ok((m, _)) =
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(&msg, bincode::config::standard())
            {
                match m {
                    RelayServerMessage::GameStarted { .. } => continue,
                    other => return other,
                }
            }
        }
    }
}

/// One round: stage due frames, flush outbound, pump inbound.
async fn pump(rs: &mut ReliableSocket) {
    rs.process();
    rs.poll().await.unwrap();
}

/// Send an unreliable heartbeat so the relay's 1.5s session sweep keeps us alive.
async fn udp_heartbeat(rs: &mut ReliableSocket) {
    rs.send_unreliable(vec![]);
    pump(rs).await;
}

/// Pump until the reconnect metadata arrives. Uses take_messages_matching so a
/// page arriving in the same batch is NOT consumed (left for the page helper).
async fn udp_recv_reconnect_meta(rs: &mut ReliableSocket, secs: u64) -> ReconnectResponse {
    let start = std::time::Instant::now();
    let mut last_hb = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(secs) {
            panic!("reconnect metadata timeout");
        }
        if last_hb.elapsed() >= Duration::from_millis(300) {
            udp_heartbeat(rs).await;
            last_hb = std::time::Instant::now();
        }
        pump(rs).await;
        let msgs = rs.take_messages_matching(|m| {
            matches!(
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(m, bincode::config::standard()),
                Ok((RelayServerMessage::ReconnectResponse(_), _))
            )
        });
        if let Some(m) = msgs.into_iter().next() {
            if let Ok((RelayServerMessage::ReconnectResponse(r), _)) =
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(&m, bincode::config::standard())
            {
                return r;
            }
        }
    }
}

/// Pump until the reconnect page with `page_index` arrives (Control-channel
/// order), sending a heartbeat every ~300ms so the relay's 1.5s sweep does not
/// kill the session while the reliable window drains (mirrors the production
/// client). Matches a SPECIFIC page so earlier pages stay buffered for their
/// own calls.
async fn udp_recv_reconnect_page(rs: &mut ReliableSocket, page_index: u32, secs: u64) -> ReconnectPage {
    let start = std::time::Instant::now();
    let mut last_hb = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(secs) {
            panic!("reconnect page {} timeout", page_index);
        }
        if last_hb.elapsed() >= Duration::from_millis(300) {
            udp_heartbeat(rs).await;
            last_hb = std::time::Instant::now();
        }
        pump(rs).await;
        let msgs = rs.take_messages_matching(|m| {
            matches!(
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(m, bincode::config::standard()),
                Ok((RelayServerMessage::ReconnectPage(p), _)) if p.page_index == page_index
            )
        });
        if let Some(m) = msgs.into_iter().next() {
            if let Ok((RelayServerMessage::ReconnectPage(p), _)) =
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(&m, bincode::config::standard())
            {
                return p;
            }
        }
    }
}

/// Pump until the BroadcastFrame for `want_tick` arrives. Uses
/// take_messages_matching so other messages (GameStarted, later broadcasts)
/// are NOT consumed — avoids the batch-discard race.
async fn udp_recv_broadcast_tick(rs: &mut ReliableSocket, want_tick: u32, secs: u64) {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(secs) {
            panic!("broadcast tick {} timeout", want_tick);
        }
        pump(rs).await;
        let msgs = rs.take_messages_matching(|m| {
            matches!(
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(m, bincode::config::standard()),
                Ok((RelayServerMessage::Broadcast(_), _))
            )
        });
        for m in msgs {
            if let Ok((RelayServerMessage::Broadcast(BroadcastFrame { payload, .. }), _)) =
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(&m, bincode::config::standard())
            {
                if payload.tick == want_tick {
                    return;
                }
            }
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_relay_reconnect_multipage() {
    let port = find_free_port().await;
    tokio::spawn(async move {
        start_relay(port, 42, 2, Some(RelayId(42))).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (mut c0, _) = udp_join(port, RelayId(42)).await;
    let (mut c1, _) = udp_join(port, RelayId(42)).await;

    // 33 ticks → 2 页(32/1);心跳保持两会话存活
    for tick in 1u32..=33 {
        udp_send_tick(&mut c0, tick, 0).await;
        udp_send_tick(&mut c1, tick, 1).await;
        udp_recv_broadcast_tick(&mut c0, tick, 5).await;
        udp_recv_broadcast_tick(&mut c1, tick, 5).await;
        udp_heartbeat(&mut c0).await;
        udp_heartbeat(&mut c1).await;
    }

    // c0 重连:last_tick_consumed=0 → 元数据 + 2 页
    let req = RelayClientMessage::Reconnect(ReconnectRequest { game_id: 1, last_tick_consumed: 0 });
    let data = bincode::serde::encode_to_vec(&req, bincode::config::standard()).unwrap();
    c0.send_reliable(CH_CONTROL, data);
    pump(&mut c0).await;

    let meta = udp_recv_reconnect_meta(&mut c0, 5).await;
    assert_eq!(meta.first_tick, 1);
    assert_eq!(meta.total_ticks, 33);
    assert_eq!(meta.page_count, 2);

    let mut seen: Vec<u32> = Vec::new();
    for i in 0..2 {
        let page = udp_recv_reconnect_page(&mut c0, i, 5).await;
        assert_eq!(page.page_index, i, "pages must arrive in Control-channel order");
        assert_eq!(page.page_count, 2);
        for b in page.ticks {
            seen.push(b.tick);
        }
    }
    assert_eq!(seen.len(), 33, "2 pages must cover all 33 ticks");
    seen.sort_unstable();
    assert_eq!(seen, (1..=33).collect::<Vec<u32>>(), "pages cover ticks 1..=33 exactly once");
}

#[tokio::test(flavor = "current_thread")]
async fn test_relay_two_clients_full_cycle() {
    let port = find_free_port().await;
    tokio::spawn(async move {
        start_relay(port, 42, 2, Some(RelayId(42))).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (mut c0, pid0) = udp_join(port, RelayId(42)).await;
    let (mut c1, pid1) = udp_join(port, RelayId(42)).await;
    assert_eq!((pid0, pid1), (0, 1));

    udp_send_tick(&mut c0, 1, 0).await;
    udp_send_tick(&mut c1, 1, 1).await;

    let b0 = udp_recv_message(&mut c0, 5).await;
    let b1 = udp_recv_message(&mut c1, 5).await;

    match (&b0, &b1) {
        (
            RelayServerMessage::Broadcast(BroadcastFrame { payload: p0, .. }),
            RelayServerMessage::Broadcast(BroadcastFrame { payload: p1, .. }),
        ) => {
            assert_eq!(p0.tick, 1, "client 0 receives tick 1");
            assert_eq!(p1.tick, 1, "client 1 receives tick 1");
            assert_eq!(p0.commands.len(), 0, "empty commands");
            assert_eq!(p1.commands.len(), 0, "empty commands");
        }
        _ => panic!("Both clients should receive Broadcast, got: {:?} {:?}", b0, b1),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_relay_correct_tick_advancement() {
    let port = find_free_port().await;
    tokio::spawn(async move {
        start_relay(port, 42, 2, Some(RelayId(42))).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (mut c0, _) = udp_join(port, RelayId(42)).await;
    let (mut c1, _) = udp_join(port, RelayId(42)).await;

    udp_send_tick(&mut c0, 5, 0).await;
    udp_send_tick(&mut c1, 5, 1).await;

    let b0 = udp_recv_message(&mut c0, 5).await;
    let b1 = udp_recv_message(&mut c1, 5).await;

    match (&b0, &b1) {
        (
            RelayServerMessage::Broadcast(BroadcastFrame { payload: p0, .. }),
            RelayServerMessage::Broadcast(BroadcastFrame { payload: p1, .. }),
        ) => {
            assert_eq!(p0.tick, 5, "tick 5 completed");
            assert_eq!(p1.tick, 5, "tick 5 completed");
        }
        _ => panic!("Expected Broadcast, got {:?} {:?}", b0, b1),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn test_relay_three_ticks_sequential() {
    let port = find_free_port().await;
    tokio::spawn(async move {
        start_relay(port, 42, 2, Some(RelayId(42))).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (mut c0, _) = udp_join(port, RelayId(42)).await;
    let (mut c1, _) = udp_join(port, RelayId(42)).await;

    for tick in 1u32..=3 {
        udp_send_tick(&mut c0, tick, 0).await;
        udp_send_tick(&mut c1, tick, 1).await;

        let msg0 = udp_recv_message(&mut c0, 5).await;
        let msg1 = udp_recv_message(&mut c1, 5).await;

        match (&msg0, &msg1) {
            (
                RelayServerMessage::Broadcast(BroadcastFrame { payload: p0, .. }),
                RelayServerMessage::Broadcast(BroadcastFrame { payload: p1, .. }),
            ) => {
                assert_eq!(p0.tick, tick, "c0 tick {}", tick);
                assert_eq!(p1.tick, tick, "c1 tick {}", tick);
            }
            _ => panic!("Expected Broadcast for tick {}, got {:?} {:?}", tick, msg0, msg1),
        }
    }
}

//! Integration test: reliable-UDP relay with simulated clients.
//!
//! Starts a relay server in-process, connects two virtual clients via the
//! reliable UDP transport, sends PlayerTickFrames, and verifies BroadcastFrames
//! arrive correctly.

use std::net::SocketAddr;
use std::time::Duration;

use bevy_adapter::discovery::{RelayId, RoomId};
use bevy_adapter::network::{
    BroadcastFrame, PlayerTickFrame, RelayClientMessage, RelayServerMessage,
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

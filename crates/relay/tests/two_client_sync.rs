//! E2E test: Two clients connected to same relay over reliable UDP.
//! Verifies both receive identical broadcasts (synchronization) and lobby flow.

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

async fn pump(rs: &mut ReliableSocket) {
    rs.process();
    rs.poll().await.unwrap();
}

/// Connect a UDP client and pump until GameJoined. Returns socket + player_id.
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
            panic!("udp_join timed out");
        }
        pump(&mut rs).await;
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

/// Send a relay client message on the control channel.
async fn udp_send(rs: &mut ReliableSocket, msg: &RelayClientMessage) {
    let data = bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap();
    rs.send_reliable(CH_CONTROL, data);
    pump(rs).await;
}

/// Pump until a relay message matching `want` arrives (skipping others).
#[derive(Clone, Copy, PartialEq)]
enum Want {
    LobbyUpdate,
    GameStarted,
}

async fn udp_recv_until(rs: &mut ReliableSocket, secs: u64, want: Want) -> RelayServerMessage {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(secs) {
            panic!("udp_recv_until timed out");
        }
        pump(rs).await;
        let msgs = rs.take_messages_matching(|m| {
            if let Ok((m2, _)) =
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(m, bincode::config::standard())
            {
                match want {
                    Want::LobbyUpdate => matches!(m2, RelayServerMessage::LobbyUpdate { .. }),
                    Want::GameStarted => matches!(m2, RelayServerMessage::GameStarted { .. }),
                }
            } else {
                false
            }
        });
        if let Some(m) = msgs.into_iter().next() {
            if let Ok((m2, _)) =
                bincode::serde::decode_from_slice::<RelayServerMessage, _>(&m, bincode::config::standard())
            {
                return m2;
            }
        }
    }
}

/// Pump until a non-GameStarted relay message arrives.
async fn udp_recv(rs: &mut ReliableSocket, secs: u64) -> RelayServerMessage {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(secs) {
            panic!("udp_recv timed out");
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

#[tokio::test(flavor = "current_thread")]
async fn test_two_clients_receive_identical_broadcasts() {
    let port = find_free_port().await;
    tokio::spawn(async move { start_relay(port, 42, 2, Some(RelayId(42))).await.unwrap(); });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (mut c0, pid0) = udp_join(port, RelayId(42)).await;
    let (mut c1, pid1) = udp_join(port, RelayId(42)).await;
    assert_eq!((pid0, pid1), (0, 1));

    udp_send(&mut c0, &RelayClientMessage::PlayerTick(PlayerTickFrame {
        magic: 0xBEEF, game_id: 1, tick: 1, player_id: 0, commands: vec![], player_sid: 1, version: 1,
    })).await;
    udp_send(&mut c1, &RelayClientMessage::PlayerTick(PlayerTickFrame {
        magic: 0xBEEF, game_id: 1, tick: 1, player_id: 1, commands: vec![], player_sid: 1, version: 1,
    })).await;

    let b0 = udp_recv(&mut c0, 5).await;
    let b1 = udp_recv(&mut c1, 5).await;

    let batch0 = match b0 {
        RelayServerMessage::Broadcast(ref b) => b.payload.clone(),
        other => panic!("c0 expected Broadcast, got {:?}", other),
    };
    let batch1 = match b1 {
        RelayServerMessage::Broadcast(ref b) => b.payload.clone(),
        other => panic!("c1 expected Broadcast, got {:?}", other),
    };

    assert_eq!(batch0.tick, batch1.tick);
    let ser0 = bincode::serde::encode_to_vec(&batch0, bincode::config::standard()).unwrap();
    let ser1 = bincode::serde::encode_to_vec(&batch1, bincode::config::standard()).unwrap();
    assert_eq!(ser0, ser1, "Both clients MUST receive identical broadcasts");
}

#[tokio::test(flavor = "current_thread")]
async fn test_two_clients_lobby_ready_then_game_started() {
    let port = find_free_port().await;
    tokio::spawn(async move { start_relay(port, 42, 2, Some(RelayId(42))).await.unwrap(); });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (mut c0, _) = udp_join(port, RelayId(42)).await;
    let (mut c1, _) = udp_join(port, RelayId(42)).await;

    // c0 sends LobbyReady → relay broadcasts LobbyUpdate
    udp_send(&mut c0, &RelayClientMessage::LobbyReady {
        game_id: 1, player_id: 0, ready: true, map_size: None,
    }).await;

    match udp_recv_until(&mut c0, 5, Want::LobbyUpdate).await {
        RelayServerMessage::LobbyUpdate { .. } => {}
        other => panic!("c0 expected LobbyUpdate, got {:?}", other),
    }

    match udp_recv_until(&mut c1, 5, Want::LobbyUpdate).await {
        RelayServerMessage::LobbyUpdate { players, .. } => {
            assert_eq!(players.len(), 2, "Expected 2 players");
            let c0_rdy = players.iter().find(|p| p.player_id == 0).map(|p| p.ready).unwrap_or(false);
            assert!(c0_rdy, "c0 should be ready after sending LobbyReady");
        }
        other => panic!("c1 expected LobbyUpdate, got {:?}", other),
    }

    // c1 sends LobbyReady → all ready → LobbyUpdate + GameStarted
    udp_send(&mut c1, &RelayClientMessage::LobbyReady {
        game_id: 1, player_id: 1, ready: true, map_size: None,
    }).await;

    match udp_recv_until(&mut c1, 5, Want::LobbyUpdate).await {
        RelayServerMessage::LobbyUpdate { .. } => {}
        other => panic!("c1 expected LobbyUpdate, got {:?}", other),
    }

    match udp_recv_until(&mut c0, 5, Want::LobbyUpdate).await {
        RelayServerMessage::LobbyUpdate { .. } => {}
        other => panic!("c0 expected LobbyUpdate, got {:?}", other),
    }

    match udp_recv_until(&mut c1, 5, Want::GameStarted).await {
        RelayServerMessage::GameStarted { player_count, .. } => {
            assert_eq!(player_count, 2);
        }
        other => panic!("c1 expected GameStarted, got {:?}", other),
    }

    match udp_recv_until(&mut c0, 5, Want::GameStarted).await {
        RelayServerMessage::GameStarted { player_count, .. } => {
            assert_eq!(player_count, 2);
        }
        other => panic!("c0 expected GameStarted, got {:?}", other),
    }
}

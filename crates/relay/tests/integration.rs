//! Integration test: TCP relay with simulated clients.
//!
//! Starts a relay server in-process, connects two virtual clients via TCP,
//! sends PlayerTickFrames, and verifies BroadcastFrames arrive correctly.

use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

use relay::start_relay;
use bevy_adapter::network::{
    BroadcastFrame, PlayerTickFrame, RelayClientMessage, RelayServerMessage,
};

/// Find a free port.

/// Helper: read message, skipping GameStarted (which arrives when both players connect)
async fn read_broadcast(stream: &mut TcpStream, secs: u64) -> RelayServerMessage {
    let msg = read_msg(stream, secs).await;
    match msg {
        RelayServerMessage::GameStarted { .. } => read_msg(stream, secs).await,
        other => other,
    }
}

async fn find_free_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind failed");
    listener.local_addr().unwrap().port()
}

/// Helper: write a relay client message to TCP.
async fn write_msg(stream: &mut TcpStream, msg: &RelayClientMessage) {
    let data = bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap();
    let len = (data.len() as u32).to_le_bytes();
    stream.write_all(&len).await.unwrap();
    stream.write_all(&data).await.unwrap();
}

/// Helper: read a relay server message from TCP.
async fn read_msg(stream: &mut TcpStream, timeout_secs: u64) -> RelayServerMessage {
    timeout(Duration::from_secs(timeout_secs), async {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.unwrap();
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await.unwrap();
        bincode::serde::decode_from_slice(&buf, bincode::config::standard())
            .unwrap()
            .0
    })
    .await
    .expect("read_msg timed out")
}

/// Helper: connect a client, expect GameJoined, send a PlayerTickFrame.
async fn connect_and_send(port: u16, tick: u32, player_id: u8) -> TcpStream {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect failed");

    // Expect GameJoined
    match read_msg(&mut stream, 5).await {
        RelayServerMessage::GameJoined { player_id: pid, .. } => {
            assert_eq!(pid, player_id, "assigned player_id must match");
        }
        other => panic!("Expected GameJoined, got {:?}", other),
    }

    // Send a frame
    let frame = PlayerTickFrame {
        magic: 0xBEEF,
            version: 1,
        game_id: 1,
        tick,
        player_id,
        commands: vec![],
        player_sid: 1,
    };
    write_msg(&mut stream, &RelayClientMessage::PlayerTick(frame)).await;

    stream
}

#[tokio::test(flavor = "current_thread")]
async fn test_relay_two_clients_full_cycle() {
    let port = find_free_port().await;

    // Start relay in background task
    tokio::spawn(async move {
        start_relay(port, 42, 2).await.unwrap();
    });

    // Allow relay to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Client 0: connect, send frame for tick 1
    let mut c0 = connect_and_send(port, 1, 0).await;

    // Client 1: connect, send frame for tick 1 → both players ready → barrier fires
    let mut c1 = connect_and_send(port, 1, 1).await;

    // Both clients should receive BroadcastFrame for tick 1
    let b0 = read_broadcast(&mut c0, 5).await;
    let b1 = read_broadcast(&mut c1, 5).await;

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
        start_relay(port, 42, 2).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut c0 = connect_and_send(port, 5, 0).await;
    let mut c1 = connect_and_send(port, 5, 1).await;

    // Both send for tick 5 → verify both get tick 5 back
    let b0 = read_broadcast(&mut c0, 5).await;
    let b1 = read_broadcast(&mut c1, 5).await;

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
        start_relay(port, 42, 2).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Connect both clients first
    let mut c0 = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let mut c1 = TcpStream::connect(("127.0.0.1", port)).await.unwrap();

    // Read GameJoined for both
    read_msg(&mut c0, 5).await;
    read_msg(&mut c1, 5).await;

    // Send frames for tick 1, 2, 3 from both players
    for tick in 1u32..=3 {
        write_msg(&mut c0, &RelayClientMessage::PlayerTick(PlayerTickFrame {
            magic: 0xBEEF, version: 1, game_id: 1, tick, player_id: 0, commands: vec![], player_sid: tick as u64,
        })).await;
        write_msg(&mut c1, &RelayClientMessage::PlayerTick(PlayerTickFrame {
            magic: 0xBEEF, version: 1, game_id: 1, tick, player_id: 1, commands: vec![], player_sid: tick as u64,
        })).await;

        // Both should get the broadcast
        let msg0 = read_broadcast(&mut c0, 5).await;
        let msg1 = read_broadcast(&mut c1, 5).await;

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

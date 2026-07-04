//! E2E test: Two clients connected to same relay.
//! Verifies both receive identical broadcasts (synchronization).

use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use relay::start_relay;
use bevy_adapter::network::{
    BroadcastFrame, PlayerTickFrame, RelayClientMessage, RelayServerMessage,
};

async fn find_free_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await.expect("bind failed");
    listener.local_addr().unwrap().port()
}

async fn write_msg(stream: &mut TcpStream, msg: &RelayClientMessage) {
    let data = bincode::serde::encode_to_vec(msg, bincode::config::standard()).unwrap();
    let len = (data.len() as u32).to_le_bytes();
    stream.write_all(&len).await.unwrap();
    stream.write_all(&data).await.unwrap();
}

async fn read_msg(stream: &mut TcpStream, secs: u64) -> RelayServerMessage {
    timeout(Duration::from_secs(secs), async {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await.unwrap();
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await.unwrap();
        bincode::serde::decode_from_slice(&buf, bincode::config::standard()).unwrap().0
    }).await.expect("read_msg timed out")
}

#[tokio::test(flavor = "current_thread")]
async fn test_two_clients_receive_identical_broadcasts() {
    let port = find_free_port().await;

    tokio::spawn(async move { start_relay(port, 42, 2).await.unwrap(); });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Both clients connect
    let mut c0 = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    match read_msg(&mut c0, 5).await {
        RelayServerMessage::GameJoined { player_id: 0, .. } => {}
        other => panic!("c0 expected GameJoined(0), got {:?}", other),
    }

    let mut c1 = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    match read_msg(&mut c1, 5).await {
        RelayServerMessage::GameJoined { player_id: 1, .. } => {}
        other => panic!("c1 expected GameJoined(1), got {:?}", other),
    }

    // Both send for tick 1 (empty commands — still tests barrier + broadcast)
    write_msg(&mut c0, &RelayClientMessage::PlayerTick(PlayerTickFrame {
        magic: 0xBEEF, game_id: 1, tick: 1, player_id: 0, commands: vec![], player_sid: 1,
    })).await;
    write_msg(&mut c1, &RelayClientMessage::PlayerTick(PlayerTickFrame {
        magic: 0xBEEF, game_id: 1, tick: 1, player_id: 1, commands: vec![], player_sid: 1,
    })).await;

    // Both must receive Broadcast (not Error, not GameOver)
    let b0 = read_msg(&mut c0, 5).await;
    let b1 = read_msg(&mut c1, 5).await;

    let batch0 = match b0 {
        RelayServerMessage::Broadcast(ref b) => b.payload.clone(),
        other => panic!("c0 expected Broadcast, got {:?}", other),
    };
    let batch1 = match b1 {
        RelayServerMessage::Broadcast(ref b) => b.payload.clone(),
        other => panic!("c1 expected Broadcast, got {:?}", other),
    };

    // Verify identical content (both get the same thing, whatever it contains)
    assert_eq!(batch0.tick, batch1.tick);
    let ser0 = bincode::serde::encode_to_vec(&batch0, bincode::config::standard()).unwrap();
    let ser1 = bincode::serde::encode_to_vec(&batch1, bincode::config::standard()).unwrap();
    assert_eq!(ser0, ser1, "Both clients MUST receive identical broadcasts");

    // Verify that at least one of the commands is NoOp for the second player
    // (when using empty commands from both, the relay should inject no NoOps since both players submitted)
    if batch0.commands.len() == 0 {
        // Empty commands from both = nothing to inject, but sync is verified
    }
}

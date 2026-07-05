//! Network mode initializer — TCP 连接 + 握手 + 返回 bootstrap facts。

use crate::session::SessionConfig;
use crate::transport::{NetworkClientHandle, NetworkReceiver, NetworkSender};

/// Network initializer 返回的 bootstrap facts。
/// 不含 CommandSource——由 wire() 统一构造。
pub struct NetworkBootstrapResult {
    pub player_id: u8,
    pub receiver: NetworkReceiver,
    pub sender: NetworkSender,
    pub handle: NetworkClientHandle,
}

/// 建立 TCP 连接，完成握手，返回 bootstrap facts。
/// 先同步阻塞确认 relay 可达，再启动异步通信线程。
pub fn initialize(config: &SessionConfig) -> Result<NetworkBootstrapResult, String> {
    let (relay_addr, _player_count, player_id) = match &config.mode {
        crate::session::SessionMode::Network { relay_addr, player_count, player_id } => {
            (relay_addr.clone(), *player_count, *player_id)
        }
        _ => return Err("Not a Network session".into()),
    };

    // Phase 1: sync TCP connect to verify relay is running
    crate::transport::connect_sync(&relay_addr)?;

    // Phase 2: spawn async client for ongoing communication
    let (receiver, sender, handle) =
        crate::transport::spawn_network_client(relay_addr, 1, player_id, 1);

    Ok(NetworkBootstrapResult {
        player_id,
        receiver,
        sender,
        handle,
    })
}

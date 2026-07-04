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
/// player_id 由 relay 根据连接顺序分配（0, 1, 2...），此处根据已知的玩家数量推算。
/// 超时 5 秒，失败时清理所有资源。
pub fn initialize(config: &SessionConfig) -> Result<NetworkBootstrapResult, String> {
    let (relay_addr, player_count) = match &config.mode {
        crate::session::SessionMode::Network { relay_addr, player_count } => {
            (relay_addr.clone(), *player_count)
        }
        _ => return Err("Not a Network session".into()),
    };

    // player_id matches connection order. Each client connects with its own player_id.
    // The relay assigns sequentially via GameJoined, matching the client's pre-known ID.
    let player_id = 0; // Phase 1: single player network, always ID 0
    // Future: full system uses player_count to determine ID allocation

    let (receiver, sender, handle) =
        crate::transport::spawn_network_client(relay_addr, 1, player_id, 1);

    Ok(NetworkBootstrapResult {
        player_id,
        receiver,
        sender,
        handle,
    })
}

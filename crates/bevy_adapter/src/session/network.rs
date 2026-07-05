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
/// 通过 spawn_network_client 异步连接（自带重试），
/// 不先进行同步连接——同步连接会注册为玩家后断开，扰乱 relay 的玩家 ID 分配。
pub fn initialize(config: &SessionConfig) -> Result<NetworkBootstrapResult, String> {
    let (relay_addr, _player_count, player_id) = match &config.mode {
        crate::session::SessionMode::Network { relay_addr, player_count, player_id } => {
            (relay_addr.clone(), *player_count, *player_id)
        }
        _ => return Err("Not a Network session".into()),
    };

    let (receiver, sender, handle) =
        crate::transport::spawn_network_client(relay_addr, 1, player_id, 1);

    Ok(NetworkBootstrapResult {
        player_id,
        receiver,
        sender,
        handle,
    })
}

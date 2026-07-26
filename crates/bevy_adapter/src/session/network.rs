//! Network mode initializer — TCP 连接 + 握手 + 返回 bootstrap facts。

use crate::discovery::RelayId;
use crate::network::NetworkEventReceiver;
use crate::session::SessionConfig;
use crate::transport::{NetworkClientHandle, NetworkReceiver, NetworkSender};

/// Network initializer 返回的 bootstrap facts。
/// 不含 CommandSource——由 wire() 统一构造。
pub struct NetworkBootstrapResult {
    pub player_id: u8,
    pub receiver: NetworkReceiver,
    pub sender: NetworkSender,
    pub handle: NetworkClientHandle,
    pub event_receiver: NetworkEventReceiver,
}

/// 建立 TCP 连接（阻塞等待连接成功），完成握手，返回 bootstrap facts。
///
/// 通过 spawn_network_client 异步连接（阻塞直到 TCP 建立，最多 30 秒），
/// 注册传输资源后返回，以便上层进入 Lobby 状态等待 GameStarted。
pub fn initialize(config: &SessionConfig) -> Result<NetworkBootstrapResult, String> {
    let (relay_addr, _player_count, player_id) = match &config.mode {
        crate::session::SessionMode::Network { relay_addr, player_count, player_id } => {
            (relay_addr.clone(), *player_count, *player_id)
        }
        _ => return Err("Not a Network session".into()),
    };

    let event_receiver = NetworkEventReceiver::default();
    let (receiver, sender, handle) =
        crate::transport::spawn_network_client(relay_addr, 1, player_id, 1, event_receiver.clone(), RelayId(0))?;

    Ok(NetworkBootstrapResult {
        player_id,
        receiver,
        sender,
        handle,
        event_receiver,
    })
}

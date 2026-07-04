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

/// 建立 TCP 连接，完成 GameJoined 握手。
/// 超时 5 秒，失败时清理所有资源。
pub fn initialize(config: &SessionConfig) -> Result<NetworkBootstrapResult, String> {
    let relay_addr = match &config.mode {
        crate::session::SessionMode::Network { relay_addr, .. } => relay_addr.clone(),
        _ => return Err("Not a Network session".into()),
    };

    let (player_id_rx, receiver, sender, handle) =
        spawn_network_client_with_game_joined(relay_addr.clone(), 1, 1)?;

    // 等待 GameJoined（同步阻塞，bootstrap 域允许）
    let player_id = player_id_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| {
            // 失败时清理：通过 drop handle 结束 tokio 线程
            handle.abort();
            "Handshake timeout: relay unreachable or no response within 5s".to_string()
        })?;

    Ok(NetworkBootstrapResult {
        player_id,
        receiver,
        sender,
        handle,
    })
}

/// 生成 NetworkClient（含 GameJoined 回传通道）。
/// transport.rs 中真正的实现，此处为其签名包装。
fn spawn_network_client_with_game_joined(
    relay_addr: String,
    game_id: u64,
    _ruleset_version: u32,
) -> Result<
    (
        std::sync::mpsc::Receiver<u8>,
        NetworkReceiver,
        NetworkSender,
        NetworkClientHandle,
    ),
    String,
> {
    // Delegate to transport.rs
    crate::transport::spawn_with_game_joined(&relay_addr, game_id)
}

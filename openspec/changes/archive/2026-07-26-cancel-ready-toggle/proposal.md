## Why

C2 实现了就绪按钮但不可取消。用户点了就绪后无法反悔。需要 relay_core 协议扩展支持取消就绪，并在 UI 中提供 toggle 行为。

## What Changes

- `RelayServer::on_lobby_not_ready(player_id)` + `is_game_started()`
- `relay_core` LobbyReady 处理：game_started 守卫 + 双向分发 + 统一 LobbyUpdate 广播
- `NetworkSender::send_lobby_ready(player_id, ready: bool)` 参数化
- Lobby UI 就绪按钮 toggle + update_ready_button 双向更新

## Capabilities

### Modified Capabilities
- `relay-lobby-protocol`: 取消就绪协议扩展
- `lobby-ui-v2`: 就绪按钮 toggle

## Impact

| 范围 | 文件 | 说明 |
|---|---|---|
| 修改 | `bevy_adapter/src/network.rs` | RelayServer.on_lobby_not_ready + is_game_started |
| 修改 | `bevy_adapter/src/relay_core.rs` | LobbyReady 处理（game_started 守卫 + 双向分发） |
| 修改 | `bevy_adapter/src/transport.rs` | send_lobby_ready 签名改为 (player_id, ready) |
| 修改 | `render_view/src/ui/lobby.rs` | 就绪按钮 toggle + update_ready_button 双向更新 |

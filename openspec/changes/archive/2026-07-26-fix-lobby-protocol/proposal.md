## Why

当前 Lobby 联机流程有三个阻塞级断裂：JoinGame 消息从未发送、房主创建房间后无法进入 Lobby、LobbyUpdate 处理错误。联机流程在"加入"阶段即断裂，需要修复底层协议后再构建完整等待页 UI。

## What Changes

- 新增 `NetworkSender::send_join_game(relay_id)` 方法
- `NeedsGameReset::Network` 增加 `relay_id: RelayId` 字段
- `handle_create_room` 成功后跳转 Lobby（设置 NeedsGameReset + GameState::Lobby）
- 引入 `IsHost(bool)` Resource 替代 player_id 隐式判断
- 修复 LobbyUpdate 处理（不再错误触发 Ready）
- 新增 `LobbyPlayerList(Vec<LobbyPlayerState>)` Resource

## Capabilities

### New Capabilities
- `lobby-update-flow`: LobbyUpdate 事件正确驱动 UI 状态 + 玩家列表数据存储

### Modified Capabilities
- `lobby-flow-cleanup`: 修复 JoinGame 发送 + 房主进入 Lobby + IsHost 资源

## Impact

| 范围 | 文件 | 说明 |
|---|---|---|
| 新增 | `bevy_adapter/src/network.rs` or transport.rs | JoinGame 发送方法 |
| 修改 | `render_view/src/lib.rs` | handle_create_room Lobby 跳转、lobby_update_system LobbyUpdate 修复 |
| 修改 | `render_view/src/lib.rs` | NeedsGameReset::Network 增加 relay_id |
| 新增 | `render_view/src/lib.rs` | IsHost, LobbyPlayerList Resource |
| 无 | Cargo.toml | 不新增依赖 |

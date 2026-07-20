## Why

当前局域网 MVP 中，客户端通过 `--player-id` 自选身份，relay 按 TCP 连接顺序分配 slot。加入流程缺乏身份验证和错误处理，Player 身份归属不清晰——这直接关联 #1（P1 命令不执行）和 #2（HUD 跨玩家影响）的根因。需要建立 **Relay-authoritative player identity**。

## What Changes

- **协议扩展**：激活 `JoinGame` 消息（添加 `room_id`/`relay_id` 字段），新增 `JoinRejected` 响应
- **Relay 逻辑**：`JoinGame` 处理器分配 `player_id`、验证身份、满员拒绝
- **客户端逻辑**：新增 `LocalPlayerIdentity` Resource，`NetworkCommandSource` 在 `GameJoined` 后创建
- **NeedsGameReset.Network 简化**：移除 `player_id` 字段
- **JoinRoomRequest** Resource + Integration System（#8 范围）
- **CLI 后门保留**：`--relay --player-id` 继续可用于调试

## Capabilities

### New Capabilities
- `join-flow`: 加入流程 — JoinRoomRequest → TCP → JoinGame → GameJoined → LocalPlayerIdentity → Lobby

### Modified Capabilities
<!-- 无现有 spec 变更 -->

## Impact

- `bevy_adapter/src/network.rs`：扩展 `RelayClientMessage::JoinGame`、新增 `RelayServerMessage::JoinRejected`、`GameJoined` 增加 `player_count`
- `bevy_adapter/src/network.rs`：`RelayServer` 需要 `on_join_game` 处理器
- `bevy_adapter/src/session_host/controller.rs`：`SessionController` 不需要改
- `relay/src/lib.rs`：`JoinGame` 从空实现改为完整处理器
- `render_view/src/lib.rs`：新增 `LocalPlayerIdentity` Resource、`JoinRoomRequest` Resource、Join Integration System
- `render_view/src/lib.rs`：`NeedsGameReset::Network` 移除 `player_id` 字段
- `render_view/src/lib.rs`：`setup_lobby_system` 改为由 `LocalPlayerIdentity` 驱动

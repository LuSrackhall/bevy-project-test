## Context

### 背景

当前 Lobby 联机流程有三个阻塞级断裂：

1. **JoinGame 消息从未发送**：`NetworkSender` 没有发送 `RelayClientMessage::JoinGame` 的方法。`transport.rs` 中 `run_session` 在 TCP 连接后直接进入读写循环，但 `relay_core::handle_client` 强制要求第一条消息是 JoinGame。收到其他消息会断开连接。因此当前加入流程完全不工作。

2. **房主无法进入 Lobby**：`handle_create_room` 成功启动 relay 后，从未设置 `NeedsGameReset::Network` 或跳转到 `GameState::Lobby`。房主创建房间后卡在 LanLobby 页面。

3. **LobbyUpdate 处理错误**：`lobby_update_system` 收到 `LobbyUpdate` 事件后无条件将 `LobbyPhase` 设为 `Ready`，实际上应该根据本地玩家的 ready 状态决定。

### 架构决策

经三份 Agent 评估（架构合规、技术方案、未来前瞻），确定房主也走 TCP 客户端路径加入自己的 relay（方案 A）。未来联机大厅也复用同一 Lobby 状态机。

## Goals / Non-Goals

**Goals：**

- 添加 JoinGame 发送支持（`NetworkSender`），修复加入流程
- 房主创建房间后自动跳转 Lobby（TCP 连接自己的 relay）
- 修复 LobbyUpdate 处理逻辑（不再错误触发 Ready）
- 引入 `IsHost` 资源（替代 player_id == 0 的隐式判断）
- `NeedsGameReset::Network` 补充 `relay_id` 字段

**Non-Goals：**

- 完整房间等待页 UI（玩家列表渲染等，属 C2）
- 取消就绪功能
- JoinRejected 事件映射
- 错误处理加固

## Decisions

### D1: 方案 A — 房主走 TCP 客户端路径

房主创建房间后，像加入者一样调用 `spawn_network_client_nonblocking` 连接到自己的 relay。join 时 player_id 设为 `Some(0)`（relay 的 next_player_id 从 0 开始），收到 GameJoined 后同步。房主和加入者共用 `lobby_update_system`。

### D2: 引入 `IsHost` 资源

新增 `render_view::IsHost(bool)` Resource。`handle_create_room` 中设为 `true`，`handle_join_room` 中设为 `false`。由 UI 用于房主特定控件显示。

### D3: JoinGame 发送时机

在 `spawn_network_client` / `spawn_network_client_nonblocking` 中，TCP 连接后、`run_session` 前，发送 `RelayClientMessage::JoinGame { room_id, relay_id }`。`relay_id` 需要通过参数链传入。

### D4: RelayId 传播链

```
handle_join_room (已有 JoinRoomRequest.relay_id)
  → NeedsGameReset::Network { relay_id, ... }
    → setup_lobby_system
      → spawn_network_client_nonblocking(..., relay_id)
        → TCP 线程发送 JoinGame(relay_id)
```

`handle_create_room` 通过 `controller.create_session(room)` 返回的 Session 获取 relay_id。

### D5: LobbyUpdate 修复

`lobby_update_system` 收到 `NetworkEvent::LobbyUpdate { players }` 后：
- 将 `players` 存入新资源 `LobbyPlayerList(Vec<LobbyPlayerState>)`
- 查找 `players` 中 `player_id == local_player_id` 的条目
- 仅当该条目 `ready == true` 时才设置 `LobbyPhase::Ready`

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|---|---|---|
| JoinGame 发送时序需确保在 `run_session` 之前 | 加入流程断裂 | 在 `spawn_network_client` 中、`run_session` 调用前发送 |
| 环回 TCP 添加了毫秒级延迟 | 房主体验 | 可忽略，与其他玩家延迟相比极小 |
| NeedsGameReset 字段增加 relay_id | 需要检查所有设置处 | 仅 `handle_join_room` 和 `handle_create_room` 两处 |

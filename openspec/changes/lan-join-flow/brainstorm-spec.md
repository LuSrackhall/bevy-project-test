## Context

当前局域网 MVP 中，客户端通过 `--relay <ip>:<port> --player-id <id>` CLI 参数手动指定身份。加入房间时，relay 按 TCP 连接到达顺序分配 `player_id`，无身份验证。`NeedsGameReset::Network` 将 `player_id` 作为启动参数传递给 `NetworkCommandSource`，导致客户端可以预设自己的身份。

已有协议消息：
- `RelayClientMessage::JoinGame(GameInitParams)` — 空实现，未使用
- `RelayServerMessage::GameJoined { game_id, player_id }` — 已定义但未正确使用
- `RelayClientMessage::LobbyReady` → `GameStarted` — 已实现

#7 已实现房间列表 UI，"加入"按钮已预留但无行为。

## Goals / Non-Goals

**Goals：**
- 定义 **Relay-authoritative player identity**：`player_id` 只能由 Relay 分配，客户端不得预设
- 实现 `JoinRoomIntent` → Integration System → TCP 连接 → `JoinGame` 协议 → `GameJoined` → `LocalPlayerIdentity`
- 激活 `RelayClientMessage::JoinGame` 空实现为完整加入协议
- 新增 `JoinRejected` 错误处理
- `setup_lobby_system` 重构为可由 Join Intent 驱动的通用连接管理器
- Relay 侧：分配 `player_id`、验证 `room_id`/`relay_id`、满员拒绝

**Non-Goals：**
- 不实现 LobbyReady 之外的大厅行为（#11 房间等待页）
- 不修改 Simulation 层的命令处理
- 不修复 #1 和 #2（但为修复提供正确身份基础）

## Invariants

### I1: Relay-authoritative player identity

`player_id` 只能由 Relay 分配。客户端不得预设、猜测或使用占位 `player_id`。`NeedsGameReset::Network` 中移除 `player_id` 字段。

### I2: Join 验证

加入握手时客户端发送 `room_id` + `relay_id`，relay 验证匹配。防止 Beacon 过期后误连到端口上已重启的另一个 relay。

### I3: 单连接单身份

一个活跃 TCP 连接只分配一个 `player_id`；`player_id` 在该 Session 生命周期内唯一；客户端不能请求指定 ID；满员时返回 `JoinRejected`。

## Decisions

### AD1: 协议扩展

```rust
// Client → Relay
RelayClientMessage::JoinGame {
    room_id: RoomId,
    relay_id: RelayId,
}

// Relay → Client (existing, now clarified)
RelayServerMessage::GameJoined {
    game_id: u64,
    player_id: u8,
    player_count: u8,  // 新增字段
}

// Relay → Client (new)
RelayServerMessage::JoinRejected {
    reason: String,
}
```

### AD2: 加入流程

```
用户点击 [加入]
  → JoinRoomIntent { room_id, relay_id, endpoint }
  → Join Integration System
  → TCP connect (复用 spawn_network_client_nonblocking)
  → Send JoinGame { room_id, relay_id }
  → Relay 验证身份 + 分配 player_id
  → GameJoined { player_id, player_count }
  → 写入 LocalPlayerIdentity Resource
  → NetworkCommandSource { player_id }
  → 进入 Lobby 等待
  → LobbyReady → GameStarted (复用现有)
```

### AD3: LocalPlayerIdentity Resource

```rust
/// Relay 分配的玩家身份。写入后不可修改。
#[derive(Resource)]
pub struct LocalPlayerIdentity {
    pub player_id: u8,
    pub player_count: u8,
}
```

替代 `NeedsGameReset::Network { player_id }`。仅在 `GameJoined` 后写入。

### AD4: NeedsGameReset.Network 简化

```rust
NeedsGameReset::Network {
    relay_addr: String,   // 仅保留连接地址
    player_count: u8,     // 保留 room 容量
    // player_id 删除 — 由 relay 分配
}
```

CLI `--relay --player-id` 后门保留用于调试，但不影响 `JoinGame` 协议。

### AD5: JoinRoomIntent

```rust
#[derive(Resource, Default)]
pub struct JoinRoomRequest {
    pub requested: bool,
    pub room_id: RoomId,
    pub relay_id: RelayId,
    pub endpoint: SocketAddr,
}
```

沿用 #7 的 Resource 模式（兼容 Bevy 0.19）。

## Risks / Trade-offs

- **[R1] 与现有 CLI 模式的兼容**：`--relay` + `--player-id` CLI 后门保留，但走独立路径不影响 Join 协议。
- **[R2] relay 的 JoinGame 当前是空实现**：需要同时修改 relay 端处理逻辑和客户端发送逻辑，两端需同步更新。
- **[R3] #1 #2 根因可能暴露**：本次设计是为修复 #1 #2 建立身份基础，但本身不修复。可能出现身份正确后 #1 #2 自然消失的情况，也可能有其他根因。

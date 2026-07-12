## Context

详见 `brainstorm-spec.md`（I1-I4 不变量，AD1-AD5 设计决策）。#8 的核心是建立 Relay-authoritative player identity。

## Goals / Non-Goals

**Goals：**
- JoinRoomRequest → TCP → JoinGame → GameJoined 完整链路
- Relay 分配 player_id + 身份验证
- LocalPlayerIdentity Resource（取代 NeedsGameReset 中的 player_id）
- setup_lobby_system 的重构
- JoinRejected 处理

**Non-Goals：**
- 不修 #1 #2（但为它们建立正确身份基础）
- 不实现房间等待页（#11）

## Decisions

### 数据流

```
LanLobby [加入] → JoinRoomRequest
     ↓
Join Integration System
     ↓
spawn_network_client_nonblocking (复用)
     ↓  TCP Connected
Send JoinGame { room_id, relay_id }
     ↓
Relay 验证 → 分配 player_id
     ↓  JoinRejected (满员/错误)
     ↓  GameJoined { player_id, player_count }
     ↓
LocalPlayerIdentity 写入
NetworkCommandSource 创建
     ↓
LobbyReady → GameStarted (复用现有)
```

### 协议实现

**RelayClientMessage::JoinGame：**
```rust
RelayClientMessage::JoinGame {
    room_id: RoomId,
    relay_id: RelayId,
}
```

**RelayServerMessage::GameJoined（增强）：**
```rust
RelayServerMessage::GameJoined {
    game_id: u64,
    player_id: u8,
    player_count: u8,  // 新增
}
```

**RelayServerMessage::JoinRejected（新增）：**
```rust
RelayServerMessage::JoinRejected {
    reason: String,
}
```

### 模块变更

- `bevy_adapter/src/network.rs` — 协议扩展
- `relay/src/lib.rs` — `JoinGame` 处理器
- `render_view/src/lib.rs` — `JoinRoomRequest`, `LocalPlayerIdentity`, Integration System
- `render_view/src/lib.rs` — `setup_lobby_system` 重构
- `src/main.rs` — CLI 路径保留但独立

### 测试策略

- relay 集成测试：验证 `JoinGame` → `GameJoined` / `JoinRejected`
- Integration 测试：验证 `JoinRoomRequest` → TCP → `LocalPlayerIdentity`
- 满员拒绝测试

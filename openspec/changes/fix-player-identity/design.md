## Context

详见 `brainstorm-spec.md`。加入者 player_id 恒为 0，因 `NetworkGameStart` 未被 `GameJoined` 事件更新。

## Goals / Non-Goals

**Goals:**
- GameJoined 事件必须更新 `NetworkGameStart`
- 去掉 `max_players` 硬编码

**Non-Goals:**
- 不重构身份系统

## Decisions

### GameJoined 更新

在 `lobby_update_system` 的 `LobbyPhase::Connected` 中 `NetworkEvent::GameJoined` 分支添加：
```rust
network_start.player_id = *player_id;
network_start.player_count = *player_count;
```

### max_players 数据流

`DiscoveryPacket` → `JoinRoomRequest.max_players` → `handle_join_room` → `NeedsGameReset`

## Risks / Trade-offs

- [Risk] setup_lobby 临时 player_id=0 窗口 → GameJoined 到达后立即覆盖

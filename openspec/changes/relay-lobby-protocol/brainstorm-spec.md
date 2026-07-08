## Context

当前 relay 协议仅支持 `JoinGame → GameStarted` 的立即开始流程。Lobby 阶段无玩家交互。三个子 Agent 审计确认了协议扩展方案，并给出 3 项修正建议。

### 相关文件

| 文件 | 当前状态 |
|------|----------|
| `crates/bevy_adapter/src/network.rs` | `RelayClientMessage` 3 变体，`RelayServerMessage` 6 变体 |
| `crates/relay/src/lib.rs` | `match` 处理 3 个 client 消息，无 lobby 逻辑 |
| `crates/bevy_adapter/src/transport.rs` | `match` 处理 6 个 server 消息 |
| `crates/relay/tests/two_client_sync.rs` | 可扩展为 lobby 协议测试 |

## Goals / Non-Goals

**Goals:**
- `RelayClientMessage` 新增 `LobbyReady` 变体
- `RelayServerMessage` 新增 `LobbyUpdate` 变体
- Relay 服务端 lobby 状态追踪（ready 状态）
- 所有玩家 ready → relay 自动广播 `GameStarted`
- relay 集成测试覆盖双玩家 lobby→ready→GameStarted
- `PlayerTickFrame` 加 `version: u16` 字段（§20 建议）
- `LobbyPlayerState.selected_map` 使用 `Option<MapSize>`（类型安全）

**Non-Goals:**
- 不改 lobby UI（下个变更）
- 不改仿真层
- 不改回放/Reconnect 格式

## Decisions

### D1：枚举尾步新增变体

```rust
// RelayClientMessage 末尾（idx=3）
LobbyReady { game_id: u64, player_id: u8, ready: bool, map_size: Option<simulation::map::MapSize> }
// RelayServerMessage 末尾（idx=6）
LobbyUpdate { game_id: u64, players: Vec<LobbyPlayerState> }
```

### D2：Relay 端 lobby 逻辑

`RelayServer` 增加 `on_lobby_ready()` 方法，tracking ready mask。当 `ready_mask == (1<<player_count)-1` → 广播 `GameStarted`。

### D3：PlayerTickFrame 版本字段

```rust
pub struct PlayerTickFrame {
    pub magic: u16,
    pub version: u16,   // NEW, default 1
    pub game_id: u64,
    ...
}
```

## Risks

| Risk | Mitigation |
|------|-----------|
| 枚举兼容性 | 尾步新增，旧方 decode Err，长度前缀保帧同步 |
| Match 穷尽 | 编译器强制，不会遗漏 |
| 协议测试 | relay 集成测试（原始 TCP）已验证可行 |

## Context

详见 brainstorm-spec.md（D1-D3 覆盖设计决策）。本文件补充实现细节。

## Decisions

### D1: RelayServer Lobby 实现

```rust
// RelayServer 新增字段
lobby_ready_mask: u8,

// 新增方法
fn on_lobby_ready(&mut self, player_id: u8) -> bool {
    self.lobby_ready_mask |= 1 << player_id;
    self.lobby_ready_mask == (1 << self.player_count) - 1
}
```

### D2: PlayerTickFrame 版本字段

magic=0xBEEF, version=1。

### D3: 测试

relay 集成测试（原始 TCP）：2 客户端 → LobbyReady → 断言 GameStarted。

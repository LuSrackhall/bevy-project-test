## Context

C2 实现了房间等待页 UI（玩家列表 + 就绪/开始按钮），但就绪后无法取消。取消就绪需要 relay_core 协议扩展和客户端 UI toggle。

### 当前瓶颈

1. `relay_core::handle_client` 中 `if !ready { continue; }` 直接跳过 ready:false 消息
2. `RelayServer.on_lobby_ready` 只做 `|=` 位或操作，没有清除路径
3. `NetworkSender::send_lobby_ready` 硬编码 `ready: true`
4. Lobby UI 就绪按钮点击后不可逆

### 三份 Agent 评估发现

- **协议层**: GameStarted 后处理 LobbyReady 会损坏 mask → 需要 `game_started` 守卫
- **客户端**: `send_lobby_ready` 应加参数而非新增方法；`update_ready_button` 需双向更新
- **宪法**: 5/5 PASS

## Goals / Non-Goals

**Goals：**
- `RelayServer::on_lobby_not_ready(player_id)` 清除 bit
- `relay_core` LobbyReady 处理：先检查 game_started，按 ready 值分发，统一广播 LobbyUpdate
- `NetworkSender::send_lobby_ready(player_id, ready: bool)` 参数化
- Lobby UI 就绪按钮 toggle + update_ready_button 双向更新

**Non-Goals：**
- 房主开始按钮 toggle（点击后不可逆，当前行为正确）
- 最大玩家数检查（`lobby_ready_mask` 为 u8，最多 8 人 — 预存限制）

## Decisions

### D1: RelayServer 协议扩展

```rust
pub fn on_lobby_not_ready(&mut self, player_id: u8) {
    self.lobby_ready_mask &= !(1 << player_id);
}

pub fn is_game_started(&self) -> bool {
    self.game_started
}
```

### D2: relay_core LobbyReady 处理

```rust
RelayClientMessage::LobbyReady { game_id, player_id, ready, map_size: _ } => {
    let mut server = ctx.server.lock().unwrap();
    // Ignore lobby changes after game has started
    if server.is_game_started() { continue; }
    if ready {
        server.on_lobby_ready(player_id);
    } else {
        server.on_lobby_not_ready(player_id);
    }
    let all_ready = server.lobby_ready_mask().count_ones() as u8 >= ctx.player_count;
    // 广播 LobbyUpdate + GameStarted (if all_ready) — 保持不变
}
```

### D3: NetworkSender 参数化

```rust
pub fn send_lobby_ready(&self, player_id: u8, ready: bool) {
    *self.lobby_ready.lock().unwrap() = Some(RelayClientMessage::LobbyReady {
        game_id: 1, player_id, ready, map_size: None,
    });
}
```

### D4: Lobby UI toggle

就绪按钮 observer 中：`let new_ready = !ready_state.0;` → 发送 `send_lobby_ready(pid, new_ready)` → `ready_state.0 = new_ready`
`update_ready_button` 中：`match (btn_type, ready_state.0)` 双向设置文本

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| GameStarted 后 LobbyReady 修改 mask | game_started 守卫 (D2) |
| lobby_ready_mask(u8) 限 8 人 | 预存问题，当前玩家数 <= 8 |
| 修改 send_lobby_ready 签名影响所有调用者 | 仅两处（房主开始 + 就绪 toggle），一并更新 |

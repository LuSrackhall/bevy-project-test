## Context

高维设计见 brainstorm-spec.md。C3 扩展 relay 协议支持取消就绪。

## Decisions

### D1: RelayServer 扩展 (network.rs)

```rust
pub fn on_lobby_not_ready(&mut self, player_id: u8) {
    self.lobby_ready_mask &= !(1 << player_id);
}
pub fn is_game_started(&self) -> bool { self.game_started }
```

### D2: relay_core LobbyReady 处理 (relay_core.rs)

```rust
RelayClientMessage::LobbyReady { game_id, player_id, ready, .. } => {
    let mut server = ctx.server.lock().unwrap();
    if server.is_game_started() { continue; }
    if ready { server.on_lobby_ready(player_id); }
    else { server.on_lobby_not_ready(player_id); }
    // 广播 LobbyUpdate (same as before)
    let all_ready = server.lobby_ready_mask().count_ones() as u8 >= ctx.player_count;
    // 广播 GameStarted if all_ready (same as before)
}
```

### D3: send_lobby_ready 参数化 (transport.rs)

```rust
pub fn send_lobby_ready(&self, player_id: u8, ready: bool) {
    *self.lobby_ready.lock().unwrap() = Some(RelayClientMessage::LobbyReady {
        game_id: 1, player_id, ready, map_size: None,
    });
}
```

更新所有调用处。

### D4: Lobby UI toggle (lobby.rs)

就绪按钮 observer: `let new_ready = !ready_state.0;` → `s.send_lobby_ready(pid, new_ready);` → `ready_state.0 = new_ready;`
update_ready_button: `match (btn_type, ready_state.0)` 双向更新文本。

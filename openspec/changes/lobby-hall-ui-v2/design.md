## Decisions

### D1: NetworkEvent 复用

```rust
pub enum NetworkEvent {
    GameStarted { game_id: u64, seed: u64, player_count: u8 },
    LobbyUpdate { game_id: u64, players: Vec<LobbyPlayerState> },
}
```

### D2: NetworkSender 扩展

```rust
lobby_ready: Arc<Mutex<Option<RelayClientMessage>>>,

pub fn send_lobby_ready(&self, player_id: u8) {
    *self.lobby_ready.lock().unwrap() = Some(RelayClientMessage::LobbyReady { ... });
}
```

write task: `if let Some(msg) = send.take_lobby_ready() { /* write to TCP */ }`

### D3: LobbyPhase 状态机

Connecting → Connected → Ready → Playing

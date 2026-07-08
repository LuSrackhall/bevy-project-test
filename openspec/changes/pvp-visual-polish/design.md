## Context

详见 brainstorm-spec.md。

## Decisions

### D1：两个辅助函数

```rust
fn is_player_faction(f: simulation::types::FactionId, lid: u8) -> bool {
    f == simulation::types::FactionId(lid)
}
fn faction_is_active_enemy(f: simulation::types::FactionId, lid: u8) -> bool {
    f != simulation::types::FactionId(lid) && (f == simulation::types::FactionId(0) || f == simulation::types::FactionId(1))
}
```

### D2：4 处 if-else 替换

保留各实体类型的颜色值不变。

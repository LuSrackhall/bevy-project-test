## Context

详见 brainstorm-spec.md。本文件补充实现细节。

## Decisions

### D1: PlayerSlots 过滤

```rust
let active_factions: Vec<FactionId> = world
    .get_resource::<PlayerSlots>()
    .map(|s| s.slots.iter()
        .filter(|s| s.controller.is_active())
        .map(|s| s.faction)
        .collect())
    .unwrap_or_default();
```

`unwrap_or_default()` 回退为空 Vec——此时所有非己方 faction 均不计入 enemy，仅 `!has_my_faction` 可能触发 GameOver。

### D2: 胜负判定循环

```rust
for (f,) in q.iter(world) {
    if f.0 == FactionId(lid) {
        has_my_faction = true;
    } else if active_factions.contains(&f.0) && f.0 != FactionId(lid) {
        has_enemy = true;
    }
}
```

三重过滤：己方→标记 my→活跃敌方→标记 enemy→FactionId(2) 中立→忽略。

### D3: 条件不变

`if !has_my_faction || !has_enemy { next_state.set(GameState::GameOver); }`

匹配单机和 PvP 2 人的"last man standing"语义。

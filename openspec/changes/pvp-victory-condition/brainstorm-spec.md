## Context

`render_view/src/lib.rs:197-216` 的 `check_victory_system` 硬编码 `FactionId(0) → has_player`、`FactionId(1) → has_enemy`。单机模式下 Player 始终是 FactionId(0)，此假设成立。联机模式下 Player 2（`LocalPlayerId(1)`）的胜负判定逻辑错误：Player 2 消灭 Player 1 后游戏异常结束。

三个子 Agent 审计确认发现额外问题：`else { has_enemy = true; }` 会捕获 `FactionId(2)` 中立城市作为 "enemy"，导致即使 AI 被消灭，中立城市仍在时游戏永不结束。

## Goals / Non-Goals

**Goals:**
- `check_victory_system` 使用 `LocalPlayerId` 动态确定本地玩家阵营
- 使用 `PlayerSlots` 资源过滤中立阵营（FactionId(2+)），只将活跃玩家阵营计入胜负判定
- 单机模式行为不变（`lid=0`）
- 联机 Player 2 胜负判定正确

**Non-Goals:**
- 不改多人扩展（3+ 玩家 FFA/组队——远期 🟢）
- 不改 `debug_shape.rs`
- 不改 GameOver UI

## Decisions

### D1: 使用 PlayerSlots 过滤活跃阵营

```rust
let world = sim_world.world_ref();
let active_factions: Vec<FactionId> = world
    .get_resource::<PlayerSlots>()
    .map(|s| s.slots.iter()
        .filter(|s| s.controller.is_active())
        .map(|s| s.faction)
        .collect())
    .unwrap_or_default();
```

过滤逻辑：`else if active_factions.contains(&f.0) && f.0 != FactionId(lid)` → 只将活跃敌方阵营记为 enemy；FactionId(2) 中立城市被忽略。

### D2: 胜负条件不变

仍使用 `!has_my_faction || !has_enemy`（任意一方全灭 → GameOver）。

## Risks / Trade-offs

| Risk | 评级 | Mitigation |
|------|------|-----------|
| PlayerSlots 不存在（回退） | 🟢 低 | `unwrap_or_default()` → `active_factions` 为空 → 所有非己方 faction 均不计为 enemy → 仅 `has_my_faction` 触发 GameOver |
| 单机兼容 | 🟢 无 | `lid=0`，`active_factions` 含 [0, 1]，行为完全一致 |
| 中立城市干扰 | ✅ 已修复 | `active_factions.contains()` 过滤掉 FactionId(2) |
| 重放模式 | 🟢 改前即存在 | `check_victory_system` 在回放中运行，改前已如此 |
| 联机 lockstep 同步 | 🟢 自动同步 | 所有客户端运行相同确定性仿真 |

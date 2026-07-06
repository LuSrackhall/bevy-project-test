## ADDED Requirements

### Requirement: AI 决策基于 PlayerSlots

AI 不再硬编码 `Faction::Enemy`。`ai_decide` 接收 `PlayerSlots` 参数，遍历所有 `Controller::AI(_)` 的 slot，为每个 AI slot 对应的 faction 生成命令。

```rust
pub fn ai_decide(world: &mut World, slots: &PlayerSlots, current_tick: u32) {
    if !current_tick.is_multiple_of(AI_TICK_INTERVAL) {
        return;
    }
    for slot in slots.iter().filter(|s| matches!(s.controller, Controller::AI(_))) {
        let faction = slot.faction;
        // 为该 faction 执行现有的 AI 决策逻辑
    }
}
```

- 函数签名从 `ai_decide(world: &mut World, current_tick: u32)` 改为 `ai_decide(world: &mut World, slots: &PlayerSlots, current_tick: u32)`
- 现有 AI 决策逻辑（移动/攻击/回城/修复等）保持不变，但 `Faction::Enemy` 引用改为 `faction`
- `RunConfig.enable_ai = false` 时跳过整个 AI 循环

#### Scenario: 单人模式 AI 行为不变

- **WHEN** `PlayerSlots` 含 1 个 AI slot（faction=FactionId(1)），且 `enable_ai = true`
- **THEN** AI 每 40 tick 为 FactionId(1) 生成命令，行为与现有 `Faction::Enemy` 相同

#### Scenario: 双 AI 分配

- **WHEN** `PlayerSlots` 含 2 个 AI slot（faction=FactionId(1) 和 FactionId(2)）
- **THEN** AI 为 FactionId(1) 和 FactionId(2) 各自生成命令

#### Scenario: enable_ai = false

- **WHEN** `enable_ai = false`，即使 `PlayerSlots` 含 AI slot
- **THEN** `ai_decide` 不执行，AI slot 无命令

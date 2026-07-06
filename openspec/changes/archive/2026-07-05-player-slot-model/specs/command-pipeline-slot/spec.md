## ADDED Requirements

### Requirement: collect_command_players 基于 PlayerSlots

`collect_command_players` 不再扫描世界中的 `FactionComponent`，改为读取 `PlayerSlots`。

```rust
fn collect_command_players(slots: &PlayerSlots) -> Vec<u8> {
    slots.iter()
        .filter(|s| !matches!(s.controller, Controller::Disabled))
        .map(|s| s.faction.0)
        .collect()
}
```

- 函数签名添加 `slots: &PlayerSlots` 参数
- `run_tick` 调用处传入 Simulation 世界的 `PlayerSlots` Resource

#### Scenario: collect_command_players 单机模式

- **WHEN** `PlayerSlots` 包含 2 个 active slot（HumanLocal + AI）
- **THEN** 返回 `vec![0, 1]`

#### Scenario: collect_command_players 排除 Disabled slot

- **WHEN** `PlayerSlots` 包含 3 个 slot，slot 2 为 `Controller::Disabled`
- **THEN** 返回 `[0, 1]`，不包含 slot 2 的 faction

### Requirement: NoOp 注入基于 PlayerSlots

`run_tick` 的 NoOp 注入从基于 Faction 枚举改为基于 `PlayerSlots`。

```rust
fn inject_noop(commands: &mut Vec<GameCommand>, slots: &PlayerSlots, tick: u32) {
    let present: HashSet<u8> = commands.iter().map(|c| c.player_id).collect();
    for slot in slots.iter().filter(|s| s.controller.is_active()) {
        if !present.contains(&slot.faction.0) {
            commands.push(GameCommand { tick, player_id: slot.faction.0, action: Action::NoOp });
        }
    }
}
```

#### Scenario: NoOp 注入单机模式

- **WHEN** `PlayerSlots` 有 2 个 active slot，只有 slot 0 产生了命令
- **THEN** slot 1 对应的 faction 被注入一个 NoOp 命令

#### Scenario: NoOp 注入全活跃

- **WHEN** `PlayerSlots` 有 2 个 slot，两者都产生了命令
- **THEN** 无 NoOp 注入

### Requirement: RunConfig.enable_ai 兼容

`RunConfig.enable_ai` 的语义从「是否执行 AI tick 系统」改为「是否允许 AI Controller 槽位生成命令」。

- `enable_ai = false` → `ai_decide` 不执行，AI slot 不产生命令（等价于该 slot 被禁用）
- `enable_ai = true` → `ai_decide` 正常执行

#### Scenario: enable_ai = false

- **WHEN** `enable_ai` 为 false
- **THEN** 即使 `PlayerSlots` 中包含 AI slot，AI 也不产生任何命令

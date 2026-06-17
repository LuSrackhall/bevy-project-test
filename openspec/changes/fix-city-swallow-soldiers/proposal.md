## Why

士兵移动目标为己方城池时，`city_interaction_system` 在士兵到达城池范围后**无条件执行 despawn**（`soldier/mod.rs:554-555`），即使城池已满血 + 满级、士兵实际未做任何贡献也会被销毁并扣减出生城人口。同时，`combat_target_system` 在攻击目标消失后会清空 `command_target`（`combat/mod.rs:131`），导致玩家下达的移动/回城意图丢失。

## What Changes

- **修复城池吞兵 bug**：`city_interaction_system` 中的 despawn 逻辑将被条件守卫，仅在士兵实际执行了治愈或升级贡献时才消耗士兵；城池满血满级时士兵停在原位不被销毁。
- **修复攻击目标消失后移动意图丢失 bug**：`combat_target_system` 在士兵从 Fighting 恢复为 Moving 时，保留 `command_target` 而非清空为 `None`。

## Capabilities

### New Capabilities

（无新增 capability）

### Modified Capabilities

- `city-interaction`: 士兵到达己方城池时的消耗行为增加条件守卫，满血满级城池不再吞兵
- `combat-fixes`: 攻击目标消失后恢复移动时保留 `command_target`

## Impact

- 影响 `simulation/src/soldier/mod.rs` 的 `city_interaction_system` 函数
- 影响 `simulation/src/combat/mod.rs` 的 `combat_target_system` 函数
- 无 API / 外部依赖变更，纯逻辑条件修复

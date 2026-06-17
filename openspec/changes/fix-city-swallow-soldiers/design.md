## Context

两个独立的条件守卫 bug，均位于 `simulation` 层纯逻辑代码中，影响士兵与城池、士兵与战斗目标的交互行为。

当前代码状态：
- `city_interaction_system`（`soldier/mod.rs:529-556`）：despawn 逻辑在 `is_targeted` 块内但不在治愈/升级条件内，导致无条件消耗。
- `combat_target_system`（`combat/mod.rs:129-132`）：敌人消失后恢复 Moving 时，`command_target` 被设为 `None`。

## Goals / Non-Goals

**Goals:**
- 满血满级城池不吞兵
- 攻击目标消失后保留玩家移动意图

**Non-Goals:**
- 不重构城池/战斗系统架构
- 不改变移动系统的到达判定（距离阈值 5）
- 不调整 overlap_resolution 碰撞规则

## Decisions

### Decision 1：用 `consumed` 标志守卫 despawn

在 `city_interaction_system` 的 `is_targeted` 块内引入 `let mut consumed = false;`，在治愈/升级分支内设为 `true`，仅当 `consumed == true` 时执行 despawn + origin_decrement。

**理由**：最小化改动，不引入新的枚举或状态机，逻辑清晰可读。`break` 保留在外层，因为无论是否消耗，士兵都不应继续匹配其他城池。

### Decision 2：保留 `command_target` 作为回退目标

`combat/mod.rs:131` 的 `Movement` 构造中，`command_target` 从 `None` 改为 `sd.cmd_target`，使士兵在战斗结束后恢复向原始目标移动。

**理由**：`command_target` 是玩家下达的原始意图（如回城、移动到某点），不应因临时的自动寻敌而丢失。`target` 字段已正确设为 `ct`（`sd.cmd_target`），此处保持一致性。

## Risks / Trade-offs

- **[风险] 士兵在城池位置累积** → overlap_resolution 已处理同位置碰撞，短期可接受；长期可加驻守行为。
- **[风险] command_target 保留后士兵可能反复进出战斗** → `seek_active` 和 `seek_range` 已控制自动寻敌范围，不会导致无限循环。

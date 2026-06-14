## Context

当前的近战攻击系统设计假设单位会先进入 `SoldierState::Fighting` 状态，然后在该状态下执行攻击。但实际上：
- 移动中的单位状态为 `Moving`
- `combat_engagement_system` 被 `force_move: true` 阻止（MoveTo 的 bug）
- 即使索敌成功设置了 `Movement.target`，到达处理也会清除它

需要重新设计近战攻击逻辑，使其不依赖状态机，而是直接基于"是否有目标在攻击范围内"来判断。

## Goals / Non-Goals

**Goals:**
- MoveTo 移动中自动攻击范围内敌人（不影响移动命令执行）
- ForceMove 不攻击（骑兵例外）
- 攻击不中断移动流程
- 保持确定性（定点数、确定性随机）

**Non-Goals:**
- 不改变弓兵行为（弓兵已有独立的攻击机制）
- 不改变索敌系统的触发条件
- 不引入新的状态枚举

## Decisions

### D1: 修复 force_move 参数

最简单的修复：`MoveTo` 传 `false`，`ForceMove` 传 `true`。

### D2: 移动中攻击 — 独立于状态机

不依赖 `SoldierState::Fighting`，而是：
1. 每个 tick，遍历所有有 `Movement.target` 的近战单位
2. 查找目标当前位置
3. 如果目标在攻击范围内且冷却归零 → 执行攻击
4. 攻击后不改变任何移动相关组件（不清除 target、不改变 state、不重置 waypoint）

这与弓兵的 `archer_attack_system` 模式一致——弓兵也不依赖状态机来决定是否攻击。

### D3: ForceMove 抑制

在移动中攻击逻辑中检查 `force_move`：
- `force_move == true` 且非骑兵 → 跳过攻击
- `force_move == true` 且骑兵 → 允许攻击（骑兵攻击不影响移动）

### D4: 冷却管理

攻击后设置 `cooldown_remaining = interval_ticks`。冷却在 `melee_attack_system` 中每 tick 递减。无论单位是否在移动，冷却都正常计时。

## Risks / Trade-offs

- **风险**：移动中攻击可能导致单位在到达目标前就击杀了沿途敌人，改变了战术行为。这是预期效果，不是 bug。
- **权衡**：独立于状态机的攻击逻辑可能与索敌系统设置的 `Fighting` 状态产生冲突。需要确保两者不互相干扰。

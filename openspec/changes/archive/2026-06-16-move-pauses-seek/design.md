## Context

玩家下达移动命令时，单位的 `force_move` 标志决定了是否跳过索敌。当前 `MoveTo` 设置 `force_move = false`（不跳过），`ForceMove` 设置 `force_move = true`（跳过）。需要让所有移动命令都暂停索敌。

## Decisions

### D1: MoveTo 也设置 force_move = true

在 `consume_commands_system` 中，`MoveTo` 和 `ForceMove` 都设置 `force_move = true`。这样 `combat_engagement_system` 会跳过这些单位的索敌逻辑。

### D2: 到达后自动恢复 force_move = false

在 `soldier_movement_system` 中，当单位到达目标点（距离 < threshold）时，如果 `SeekStance.active == true`，将 `force_move` 恢复为 `false`。这样到达后索敌自动恢复。

如果 `SeekStance.active == false`（索敌关闭），到达后 `force_move` 保持 `true`，不影响行为。

### D3: 自动攻击不受影响

`melee_attack_system` 和 `archer_attack_system` 独立于 `force_move`，单位路过攻击范围内敌人时仍会自动出手。这符合"索敌暂停，自动攻击保留"的需求。

### D4: 默认范围改为 0

`SeekPanelState` 的 `range_value` 默认值从 10 改为 0。模式切换时的默认值从 30/10 改为 0。索敌默认关闭。

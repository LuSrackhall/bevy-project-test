## ADDED Requirements

### Requirement: SeekStance 组件定义

每个士兵实体 SHALL 携带 `SeekStance` 组件，包含 `active: bool` 和 `seek_range: u32` 两个字段。新生成单位的 `SeekStance` SHALL 默认为 `{ active: false, seek_range: 0 }`，除非被全局指令覆盖。

#### Scenario: 新生成单位默认不索敌

- **WHEN** `city_spawn_system` 创建新士兵，且无活跃的 `GlobalSeekDirective` 覆盖该兵种
- **THEN** 该士兵的 `SeekStance = { active: false, seek_range: 0 }`

#### Scenario: 组件可被命令更新

- **WHEN** 消费 `Action::SetSeekStance` 命令，目标为某 UnitId
- **THEN** 该 UnitId 的 `SeekStance` 被更新为命令中指定的 `active` 和 `seek_range`

### Requirement: combat_engagement_system 基于 SeekStance 决策

`combat_engagement_system` SHALL 仅对 `SeekStance.active == true` 的单位执行主动索敌。SHALL 使用 `SeekStance.seek_range` 作为索敌距离（替代已移除的 `aggression_range`）。`force_move == true` 的单位 SHALL 跳过主动索敌（与现有行为一致）。

#### Scenario: 索敌关闭时不主动移动

- **WHEN** 士兵 `SeekStance.active == false`，攻击范围外有敌人在 seek_range 内
- **THEN** `combat_engagement_system` 不修改该士兵的 `Movement.target`

#### Scenario: 索敌开启时在范围内索敌

- **WHEN** 士兵 `SeekStance = { active: true, seek_range: 150 }`，最近敌人在 100 单位处
- **THEN** `combat_engagement_system` 将该士兵的 `Movement.target` 设为该敌人，`SoldierState` 设为 `Fighting`

#### Scenario: 索敌开启但敌人超出范围

- **WHEN** 士兵 `SeekStance = { active: true, seek_range: 50 }`，最近敌人在 120 单位处
- **THEN** `combat_engagement_system` 不修改该士兵的 `Movement.target`（超出 seek_range）

#### Scenario: force_move 跳过索敌

- **WHEN** 士兵 `Movement.force_move == true`，即使 `SeekStance.active == true`
- **THEN** `combat_engagement_system` 不修改该士兵的 `Movement.target`

#### Scenario: 攻击范围内自动攻击不受 SeekStance 影响

- **WHEN** 士兵 `SeekStance.active == false`，但敌人在 `attack_range`（如 30）内
- **THEN** `melee_attack_system` 仍可正常发起攻击（不依赖 SeekStance）

### Requirement: 索敌状态切换时的 Movement 清理

当士兵的 `SeekStance.active` 从 `true` 变为 `false`，且士兵当前 `Movement.target` 是由索敌系统设置的（非玩家命令），SHALL 清除 `Movement.target`，士兵停在原地。

#### Scenario: 关闭索敌后士兵停止追击

- **WHEN** 全局索敌指令关闭（`active = false`），士兵此前因索敌正在追击敌人
- **THEN** 士兵的 `Movement.target` 被清除，`SoldierState` 恢复为 `Moving`，士兵停在当前坐标

## MODIFIED Requirements

### Requirement: 移动命令暂停索敌

`consume_commands_system` 处理 `MoveTo` 和 `ForceMove` 时 SHALL 设置 `force_move = true`。`combat_engagement_system` 在 `force_move == true` 时 SHALL 跳过该单位的索敌逻辑。

#### Scenario: 普通移动暂停索敌

- **WHEN** 士兵 `SeekStance = { active: true, seek_range: 100 }`，玩家下达 `MoveTo` 命令
- **THEN** 士兵 `force_move` 设为 `true`
- **AND** `combat_engagement_system` 跳过该士兵，不设置 `Movement.target` 为敌人
- **AND** 士兵径直走向目标点

#### Scenario: 到达后恢复索敌

- **WHEN** 士兵到达 `MoveTo` 目标点（距离 < threshold）
- **AND** 士兵 `SeekStance.active == true`
- **THEN** 士兵 `force_move` 恢复为 `false`
- **AND** 下一 tick `combat_engagement_system` 正常处理该士兵的索敌

#### Scenario: 索敌关闭时不恢复

- **WHEN** 士兵到达目标点，`SeekStance.active == false`
- **THEN** `force_move` 保持 `true`（无影响，因为索敌本身已关闭）

### Requirement: 移动途中自动攻击保留

`melee_attack_system` SHALL 独立于 `force_move`，单位路过攻击范围内敌人时 SHALL 自动攻击。

#### Scenario: 移动途中路过敌人

- **WHEN** 士兵正在移动（`force_move = true`），敌人进入 `attack_range`（30 单位）
- **THEN** 士兵自动攻击该敌人
- **AND** 士兵不停下，继续移动

### Requirement: 索敌默认范围为 0

`SeekPanelState.range_value` 默认值 SHALL 为 0。模式切换时的默认值 SHALL 为 0。

#### Scenario: 默认索敌关闭

- **WHEN** 用户打开索敌面板
- **THEN** 范围输入框显示"0"
- **AND** 下发命令 `seek_range = 0` 等于关闭索敌

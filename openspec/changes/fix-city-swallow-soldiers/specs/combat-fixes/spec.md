## ADDED Requirements

### Requirement: Preserve command_target when reverting from Fighting to Moving
当士兵处于 Fighting 状态且攻击目标消失（范围内无敌方单位）时，恢复为 Moving 状态 SHALL 保留 `command_target` 为玩家原始下达的目标，而非清空为 None。

#### Scenario: Soldier loses attack target and has command_target
- **WHEN** 士兵处于 Fighting 状态，`seek_range` 内无敌方单位，且 `command_target` 非空（如玩家下达了回城或移动指令）
- **THEN** 士兵状态恢复为 Moving，`target` 设为 `command_target` 的值，`command_target` 保持原值不清空

#### Scenario: Soldier loses attack target and has no command_target
- **WHEN** 士兵处于 Fighting 状态，`seek_range` 内无敌方单位，且 `command_target` 为 None
- **THEN** 士兵状态恢复为 Moving，`target` 设为 None（与当前行为一致）

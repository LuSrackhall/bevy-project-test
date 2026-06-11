## ADDED Requirements

### Requirement: Three display modes
系统 SHALL 提供三种信息条显示模式：Always（始终显示所有单位）、Selected（仅显示选中单位）、Smart（选中单位始终显示，未选中单位仅在有变化时显示）。

#### Scenario: Always mode
- **WHEN** 当前模式为 Always
- **THEN** 所有拥有 Health 和 Level 组件的可见单位头顶均显示信息条

#### Scenario: Selected mode
- **WHEN** 当前模式为 Selected，玩家选中了一个单位
- **THEN** 仅选中单位头顶显示信息条，未选中单位不显示

#### Scenario: Smart mode - selected unit
- **WHEN** 当前模式为 Smart，玩家选中了一个单位
- **THEN** 该单位始终显示信息条（无论血量和经验值状态）

#### Scenario: Smart mode - unselected unit with full health and zero exp
- **WHEN** 当前模式为 Smart，一个未选中单位 HP 满值且 EXP = 0
- **THEN** 该单位不显示信息条

#### Scenario: Smart mode - unselected unit with partial health
- **WHEN** 当前模式为 Smart，一个未选中单位 HP 当前值 < 最大值
- **THEN** 该单位显示信息条

#### Scenario: Smart mode - unselected unit with experience
- **WHEN** 当前模式为 Smart，一个未选中单位 EXP > 0
- **THEN** 该单位显示信息条

### Requirement: Default display mode
系统 SHALL 在游戏启动时将显示模式初始化为 Smart 模式。

#### Scenario: Game starts with Smart mode
- **WHEN** 游戏进入 Playing 状态
- **THEN** UnitInfoBarSettings.mode 值为 Smart

### Requirement: Keyboard shortcut to cycle modes
系统 SHALL 响应 Ctrl+H 快捷键，在 Always → Selected → Smart → Always 之间循环切换显示模式。

#### Scenario: Cycle from Smart to Always
- **WHEN** 当前模式为 Smart，玩家按下 Ctrl+H
- **THEN** 模式切换为 Always

#### Scenario: Cycle from Always to Selected
- **WHEN** 当前模式为 Always，玩家按下 Ctrl+H
- **THEN** 模式切换为 Selected

#### Scenario: Cycle from Selected to Smart
- **WHEN** 当前模式为 Selected，玩家按下 Ctrl+H
- **THEN** 模式切换为 Smart

### Requirement: Mode change affects all existing info bars
切换显示模式时，系统 SHALL 在下一帧立即更新所有现有信息条的可见性。

#### Scenario: Switch from Always to Selected
- **WHEN** 当前模式为 Always，所有单位均显示信息条，玩家按下 Ctrl+H 切换为 Selected
- **THEN** 下一帧仅选中单位的信息条保持可见，其余隐藏

### Requirement: Info bar visibility state restoration
从隐藏模式切换回可见模式时，系统 SHALL 恢复信息条的正常显示状态（包含正确的条宽度和数值文字）。

#### Scenario: Smart mode hides then shows bar
- **WHEN** 一个满血单位在 Smart 模式下隐藏信息条，随后受伤导致需要重新显示
- **THEN** 信息条以正确的填充宽度和数值重新显示

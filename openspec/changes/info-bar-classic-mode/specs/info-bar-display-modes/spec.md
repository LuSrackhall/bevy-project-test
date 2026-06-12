## ADDED Requirements

### Requirement: Classic display mode
系统 SHALL 提供第四种信息条显示模式 Classic：仅选中或光标悬停的单位显示血条，不考虑受伤或经验变化。

#### Scenario: Classic mode - selected unit
- **WHEN** 当前模式为 Classic，玩家选中了一个单位
- **THEN** 该单位显示血条

#### Scenario: Classic mode - hovered unit
- **WHEN** 当前模式为 Classic，光标悬停在一个未选中的单位上
- **THEN** 该单位显示血条

#### Scenario: Classic mode - unselected unhovered unit with damage
- **WHEN** 当前模式为 Classic，一个未选中且未悬停的单位 HP < max
- **THEN** 该单位不显示血条

#### Scenario: Classic mode - unselected unhovered unit with full health
- **WHEN** 当前模式为 Classic，一个未选中且未悬停的单位 HP = max
- **THEN** 该单位不显示血条

### Requirement: Hover detection for all unit types
悬停检测 SHALL 适用于所有有血条的单位，包括士兵和城池。

#### Scenario: Hover over soldier
- **WHEN** 光标悬停在一个士兵单位上
- **THEN** 该士兵显示血条（在 Selected 或 Classic 模式下）

#### Scenario: Hover over city
- **WHEN** 光标悬停在一个城池上
- **THEN** 该城池显示血条（在 Selected 或 Classic 模式下）

## MODIFIED Requirements

### Requirement: Selected display mode includes hover
Selected 模式 SHALL 在原有"仅选中"逻辑基础上增加悬停显示血条。

#### Scenario: Selected mode - selected unit
- **WHEN** 当前模式为 Selected，玩家选中了一个单位
- **THEN** 该单位显示血条

#### Scenario: Selected mode - hovered unit
- **WHEN** 当前模式为 Selected，光标悬停在一个未选中的单位上
- **THEN** 该单位显示血条

#### Scenario: Selected mode - unselected unhovered unit
- **WHEN** 当前模式为 Selected，一个未选中且未悬停的单位
- **THEN** 该单位不显示血条

### Requirement: Mode cycle order
`Ctrl+H` SHALL 按 Always → Selected → Smart → Classic → Always 顺序循环切换模式。

#### Scenario: Cycle from Smart to Classic
- **WHEN** 当前模式为 Smart，玩家按下 Ctrl+H
- **THEN** 模式切换为 Classic

#### Scenario: Cycle from Classic to Always
- **WHEN** 当前模式为 Classic，玩家按下 Ctrl+H
- **THEN** 模式切换为 Always

### Requirement: Default display mode
系统 SHALL 在游戏启动时将显示模式初始化为 Classic。

#### Scenario: Game starts with Classic mode
- **WHEN** 游戏进入 Playing 状态
- **THEN** UnitInfoBarSettings.mode 值为 Classic

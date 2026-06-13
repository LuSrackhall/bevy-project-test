## ADDED Requirements

### Requirement: 下拉框包含民兵选项

下拉选择框 SHALL 包含"民兵"选项，对应 `SeekScope::ByType(SoldierType::Militia)`。选项完整列表为：全体、民兵、步兵、弓兵、骑兵。

#### Scenario: 民兵选项可选

- **WHEN** 用户展开下拉框
- **THEN** 显示 5 个选项：全体、民兵、步兵、弓兵、骑兵
- **WHEN** 用户点击"民兵"
- **THEN** scope 更新为 `ByType(Militia)`，触发器显示"民兵 ▼"

### Requirement: 范围输入框实时生效

范围输入框 SHALL 不需要 Enter 确认。键盘数字键 SHALL 直接修改 `range_value`。Backspace SHALL 删除末位数字。输入框显示值 SHALL 始终等于 `range_value`。

#### Scenario: 输入数字立即生效

- **WHEN** 当前 range_value 为 10，用户按 3、0
- **THEN** range_value 先变为 103，再变为 1030，输入框实时显示

#### Scenario: Backspace 删除末位

- **WHEN** range_value 为 150，用户按 Backspace
- **THEN** range_value 变为 15，输入框显示"15"

#### Scenario: 全部删除后保持 0

- **WHEN** range_value 为 5，用户按 Backspace
- **THEN** range_value 变为 0，输入框显示"0"

### Requirement: 模式标签

scope 选择框左侧 SHALL 显示模式标签。无选中单位时显示"索敌"。有选中单位时显示"选中"。

#### Scenario: 全局模式标签

- **WHEN** 无单位选中
- **THEN** 标签显示"索敌"

#### Scenario: 选择模式标签

- **WHEN** 选中 3 个单位
- **THEN** 标签显示"选中"

### Requirement: 下拉选项实时数量

下拉选项 SHALL 显示对应兵种的实时数量。格式为"兵种名 (数量)"。全体选项显示总人数。

#### Scenario: 全局模式显示全军数量

- **WHEN** 全局模式，Player 有 3 民兵、5 步兵、2 弓兵、4 骑兵
- **THEN** 选项显示：全体 (14)、民兵 (3)、步兵 (5)、弓兵 (2)、骑兵 (4)

#### Scenario: 选择模式显示选中数量

- **WHEN** 选择模式，选中 5 个单位：2 步兵、3 骑兵
- **THEN** 选项显示：全体 (5)、民兵 (0)、步兵 (2)、弓兵 (0)、骑兵 (3)

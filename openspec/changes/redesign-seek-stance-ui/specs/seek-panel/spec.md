## ADDED Requirements

### Requirement: 索敌面板布局与模式切换

底部工具栏右侧 SHALL 显示索敌配置面板，由分隔线与左侧工具按钮隔开。面板 SHALL 包含三个元素：兵种选择框、范围输入框、下发按钮。面板 SHALL 根据 `SelectionState.selected_unit_ids` 是否为空自动切换模式：
- 为空时显示全局模式
- 非空时显示选择模式

两种模式下兵种选择框和下发按钮均可见，仅默认值和命令语义不同。

#### Scenario: 无选中单位时显示全局模式

- **WHEN** `SelectionState.selected_unit_ids` 为空
- **THEN** 索敌面板显示兵种选择框（默认"全体"）、范围输入框（默认值 10）、下发按钮

#### Scenario: 选中单位时显示选择模式

- **WHEN** `SelectionState.selected_unit_ids` 非空
- **THEN** 索敌面板显示兵种选择框（默认"全体"）、范围输入框（默认值 30）、下发按钮

### Requirement: 兵种下拉选择框

面板中的兵种选择框 SHALL 显示当前选中的 scope 文字和展开指示符（"▼"）。点击选择框 SHALL 展开选项列表，包含"全体"、"步兵"、"弓兵"、"骑兵"四项。点击选项 SHALL 更新当前 scope 并收起列表。点击选择框外区域 SHALL 收起列表且不修改当前选择。

选项与 `SeekScope` 的映射：
- "全体" → `SeekScope::All`
- "步兵" → `SeekScope::ByType(SoldierType::Infantry)`
- "弓兵" → `SeekScope::ByType(SoldierType::Archer)`
- "骑兵" → `SeekScope::ByType(SoldierType::Cavalry)`

#### Scenario: 展开下拉列表

- **WHEN** 用户点击兵种选择框
- **THEN** 选项列表展开，显示 4 个选项，当前选中项高亮

#### Scenario: 选择兵种选项

- **WHEN** 用户点击选项列表中的"步兵"
- **THEN** 选择框文字更新为"步兵▼"，列表收起，`scope` 更新为 `ByType(Infantry)`

#### Scenario: 点击外部收起列表

- **WHEN** 下拉列表展开状态，用户点击面板外任意位置
- **THEN** 列表收起，`scope` 保持不变

### Requirement: 范围输入框

面板中的范围输入框 SHALL 显示当前范围数值。点击输入框 SHALL 进入编辑模式。编辑模式下：
- 数字键（0-9）SHALL 追加对应字符到输入缓冲区
- Backspace SHALL 删除缓冲区末尾字符
- Enter SHALL 将缓冲区解析为 u32 并更新范围值，退出编辑模式
- Escape SHALL 丢弃缓冲区修改，恢复原值，退出编辑模式

输入缓冲区最大长度 SHALL 为 4 位（最大 9999）。空缓冲区按 Enter 时不更新。解析结果为 0 时视为关闭索敌。

#### Scenario: 点击进入编辑模式

- **WHEN** 用户点击范围输入框
- **THEN** 进入编辑模式，显示当前值的数字字符，视觉边框高亮

#### Scenario: 键盘输入数字

- **WHEN** 编辑模式下用户依次按下 `1`、`5`、`0`
- **THEN** 输入框显示"150"，缓冲区为"150"

#### Scenario: Backspace 删除字符

- **WHEN** 编辑模式下缓冲区为"150"，用户按下 Backspace
- **THEN** 输入框显示"15"，缓冲区为"15"

#### Scenario: Enter 确认输入

- **WHEN** 编辑模式下缓冲区为"30"，用户按下 Enter
- **THEN** 范围值更新为 30，退出编辑模式，输入框显示"30"

#### Scenario: Escape 取消编辑

- **WHEN** 编辑模式下缓冲区为"999"（原值为 10），用户按下 Escape
- **THEN** 范围值保持 10 不变，退出编辑模式，输入框显示"10"

#### Scenario: 空缓冲区不更新

- **WHEN** 编辑模式下用户清空所有字符后按 Enter
- **THEN** 范围值保持原值不变，退出编辑模式

### Requirement: 下发命令

点击「下发」按钮 SHALL 根据当前模式生成 `Action::SetSeekStance` 命令并推入 `CommandBuffer`。

- 全局模式：`SetSeekStance { scope: 当前scope, seek_range: 输入范围, unit_ids: [] }`
- 选择模式：`SetSeekStance { scope: 当前scope, seek_range: 输入范围, unit_ids: 选中的UnitId列表 }`

下发后 SHALL 触发 Toast 消息显示。

#### Scenario: 全局模式下发全体索敌

- **WHEN** 全局模式，scope 为"全体"，范围为 30，点击下发
- **THEN** 生成 `SetSeekStance { scope: All, seek_range: 30, unit_ids: [] }` 并推入 CommandBuffer
- **AND** 触发 Toast 消息"已下发全体索敌 范围30"

#### Scenario: 选择模式下发按兵种索敌

- **WHEN** 选择模式，选中 3 个骑兵，scope 为"骑兵"，范围为 40，点击下发
- **THEN** 生成 `SetSeekStance { scope: ByType(Cavalry), seek_range: 40, unit_ids: [选中的3个UnitId] }` 并推入 CommandBuffer
- **AND** 触发 Toast 消息"已下发选中骑兵(3)索敌 范围40"

#### Scenario: 范围为 0 等于关闭索敌

- **WHEN** 全局模式，范围为 0，点击下发
- **THEN** 生成 `SetSeekStance { scope: All, seek_range: 0, unit_ids: [] }` 并推入 CommandBuffer
- **AND** 触发 Toast 消息"已下发全体索敌 范围0"

### Requirement: 编辑模式拦截快捷键

当索敌面板范围输入框处于编辑模式时，其他快捷键系统（如 S 键索敌快捷键、Ctrl+A 全选等）SHALL 不响应键盘输入，避免冲突。

#### Scenario: 编辑模式中 S 键不触发索敌

- **WHEN** 范围输入框处于编辑模式，用户按下 S 键
- **THEN** S 键字符追加到输入缓冲区（如果有效），`seek_stance_shortcut_system` 不触发

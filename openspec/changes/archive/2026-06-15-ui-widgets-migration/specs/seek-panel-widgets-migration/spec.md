## ADDED Requirements

### Requirement: 下拉菜单使用 MenuPopup

Seek Panel 的作用域下拉菜单 SHALL 使用 `MenuPopup` + `MenuButton` + `MenuItem` 替代手写 dropdown 实现。

#### Scenario: 下拉菜单打开

- **WHEN** 玩家点击作用域下拉按钮
- **THEN** `MenuPopup` SHALL 自动显示选项列表

#### Scenario: 选项选择

- **WHEN** 玩家选择一个选项（如"步兵"）
- **THEN** `MenuEvent` SHALL 触发，Observer 更新 `SeekPanelState.scope`

#### Scenario: 点击外部关闭

- **WHEN** 玩家点击下拉菜单外部区域
- **THEN** `MenuPopup` SHALL 自动关闭

### Requirement: 下发按钮使用 Activate Observer

Seek Panel 的下发按钮 SHALL 使用 `bevy_ui_widgets::Button` + `Activate` Observer。

#### Scenario: 下发按钮点击

- **WHEN** 玩家点击下发按钮
- **THEN** `Activate` 事件 SHALL 触发，Observer 生成 `SetSeekStance` 命令

### Requirement: 范围输入框 Observer 化

范围输入框 SHALL 保留手写数字捕获逻辑，但用 Observer 替代 `Interaction` 轮询。

#### Scenario: 输入框激活

- **WHEN** 玩家点击范围输入框
- **THEN** `Pointer<Press>` Observer SHALL 激活键盘输入模式

#### Scenario: 数字输入

- **WHEN** 输入框激活后玩家按数字键
- **THEN** 系统 SHALL 接受 0-9 的数字输入，上限 4 位

#### Scenario: 输入框失活

- **WHEN** 玩家按 Escape 或点击输入框外部
- **THEN** 键盘输入模式 SHALL 关闭

### Requirement: SeekPanelState 简化

迁移完成后，`SeekPanelState` SHALL 移除不再需要的字段。

#### Scenario: dropdown_open 移除

- **WHEN** 下拉菜单迁移到 MenuPopup
- **THEN** `SeekPanelState.dropdown_open` 字段 SHALL 被移除

#### Scenario: input_active 保留

- **WHEN** 输入框保留手写逻辑
- **THEN** `SeekPanelState.input_active` 字段 SHALL 保留

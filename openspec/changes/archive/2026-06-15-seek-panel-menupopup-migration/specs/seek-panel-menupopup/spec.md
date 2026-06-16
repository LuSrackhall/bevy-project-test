## ADDED Requirements

### Requirement: 下拉菜单使用 MenuPopup 实现

Seek Panel 的作用域下拉菜单 SHALL 使用 `MenuPopup` + `MenuItem` 替代 `Display::None/Flex` 切换。

#### Scenario: 下拉菜单打开

- **WHEN** 玩家点击作用域触发按钮
- **THEN** `MenuPopup` SHALL 通过 spawn 创建，选项列表可见

#### Scenario: 选项选择

- **WHEN** 玩家点击一个选项（如"步兵"）
- **THEN** `Activate` 事件 SHALL 触发，Observer 更新 `SeekPanelState.scope`
- **THEN** MenuPopup 自动关闭（焦点回到触发按钮）

#### Scenario: 点击外部关闭

- **WHEN** 玩家点击下拉菜单外部区域
- **THEN** MenuPopup SHALL 通过焦点系统自动关闭

### Requirement: 注册必要插件

系统 SHALL 注册 `MenuPlugin` + `PopoverPlugin` + `TabNavigationPlugin` + `InputDispatchPlugin`。

#### Scenario: 插件注册

- **WHEN** 应用启动
- **THEN** 所有必要插件 SHALL 已注册，`InputFocus` 资源已初始化

### Requirement: 删除旧的下拉菜单实现

旧的 `seek_panel_dropdown_system` 和相关 workaround SHALL 被删除。

#### Scenario: 旧系统删除

- **WHEN** 迁移完成
- **THEN** `seek_panel_dropdown_system` 函数 SHALL 被删除
- **THEN** `SeekPanelState.dropdown_open` 和 `trigger_clicked` 字段 SHALL 被删除
- **THEN** `SeekDropdownPopup` 组件 SHALL 被删除

### Requirement: 输入框键盘捕获不受影响

范围输入框的键盘捕获逻辑 SHALL 保持不变。

#### Scenario: 菜单关闭后输入框正常工作

- **WHEN** 下拉菜单关闭，且输入框处于激活状态
- **THEN** 数字键输入 SHALL 正常响应

## ADDED Requirements

### Requirement: UI 点击不穿透到游戏世界

当光标位于 UI 元素上方时，左键点击 SHALL NOT 触发游戏世界的选中/取消选中逻辑。

#### Scenario: 点击 UI 按钮时选中状态保持

- **WHEN** 玩家已选中游戏单位，且光标位于 UI 按钮上方
- **WHEN** 玩家左键点击该 UI 按钮
- **THEN** 游戏世界的选中状态 SHALL 保持不变，UI 按钮功能 SHALL 正常触发

#### Scenario: 点击游戏世界空白处时清除选中

- **WHEN** 玩家已选中游戏单位，且光标位于游戏世界空白处（无 UI 覆盖）
- **WHEN** 玩家左键点击
- **THEN** 游戏世界的选中状态 SHALL 被清除

### Requirement: 右键点击 UI 不触发游戏命令

当光标位于 UI 元素上方时，右键点击 SHALL NOT 触发 `command_issue_system` 的游戏命令逻辑。

#### Scenario: 右键点击 UI 按钮时不触发移动/攻击命令

- **WHEN** 玩家已选中游戏单位，且光标位于 UI 按钮上方
- **WHEN** 玩家右键点击该 UI 按钮
- **THEN** `command_issue_system` SHALL NOT 生成任何游戏命令

### Requirement: 使用 Interaction::Pressed 检测 UI 交互状态

系统 SHALL 使用 `Interaction::Pressed` 组件状态判断是否有 UI 元素正在被按下，替代手动 `UiFocusBlocker` 机制。

#### Scenario: 按钮被按下时 Interaction::Pressed 为 true

- **WHEN** 玩家按下 UI 按钮
- **THEN** 该按钮的 `Interaction` 组件 SHALL 变为 `Pressed` 状态

#### Scenario: 透明容器不会产生 Interaction::Pressed

- **WHEN** 光标位于透明容器上方（无 BackgroundColor）
- **THEN** 透明容器的 `Interaction` 组件 SHALL NOT 为 `Pressed`（仅有 `Hovered`）

#### Scenario: 选择系统检查 Interaction::Pressed

- **WHEN** `selection_click_system` 执行
- **THEN** 系统 SHALL 通过 `Query<&Interaction>` 检查是否有任何 UI 元素处于 `Pressed` 状态

### Requirement: 删除 UiFocusBlocker 机制

`UiFocusBlocker` 资源、`reset_ui_focus_blocker` 系统及其所有引用 SHALL 被删除。

#### Scenario: UiFocusBlocker 资源不存在

- **WHEN** 代码迁移完成
- **THEN** `UiFocusBlocker` 结构体定义、`reset_ui_focus_blocker` 函数、`.init_resource::<UiFocusBlocker>()` 注册 SHALL 全部被移除

#### Scenario: seek panel 系统无 blocker 参数

- **WHEN** `seek_panel_dropdown_system`、`seek_panel_input_system`、`seek_panel_issue_system` 执行
- **THEN** 这些系统 SHALL NOT 依赖 `ResMut<UiFocusBlocker>` 参数

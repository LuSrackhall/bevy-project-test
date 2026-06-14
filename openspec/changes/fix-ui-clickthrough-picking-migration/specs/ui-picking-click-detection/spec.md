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

### Requirement: 使用 Picking 系统 HoverMap 检测 UI 状态

系统 SHALL 使用 Bevy Picking 系统的 `HoverMap` 资源判断光标是否在 UI 上，替代手动 `UiFocusBlocker` 机制。

#### Scenario: HoverMap 在 selection_click_system 中可用

- **WHEN** `selection_click_system` 执行
- **THEN** 系统 SHALL 能够通过 `Res<HoverMap>` 查询当前鼠标指针的 hover 状态

#### Scenario: 光标在 UI 上时 HoverMap 非空

- **WHEN** 光标位于任何可见 UI 节点上方
- **THEN** `HoverMap` 中 SHALL 包含鼠标指针的实体条目

#### Scenario: 光标在空白处时 HoverMap 为空

- **WHEN** 光标不位于任何 UI 节点上方
- **THEN** `HoverMap` 中鼠标指针的条目 SHALL 为空或不存在

### Requirement: 删除 UiFocusBlocker 机制

`UiFocusBlocker` 资源、`reset_ui_focus_blocker` 系统及其所有引用 SHALL 被删除。

#### Scenario: UiFocusBlocker 资源不存在

- **WHEN** 代码迁移完成
- **THEN** `UiFocusBlocker` 结构体定义、`reset_ui_focus_blocker` 函数、`.init_resource::<UiFocusBlocker>()` 注册 SHALL 全部被移除

#### Scenario: seek panel 系统无 blocker 参数

- **WHEN** `seek_panel_dropdown_system`、`seek_panel_input_system`、`seek_panel_issue_system` 执行
- **THEN** 这些系统 SHALL NOT 依赖 `ResMut<UiFocusBlocker>` 参数

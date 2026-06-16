## ADDED Requirements

### Requirement: 按钮使用 bevy_ui_widgets::Button

所有按钮 SHALL 使用 `bevy_ui_widgets::Button` 替代 `bevy_ui::widget::Button`。

#### Scenario: menu 按钮迁移

- **WHEN** 主菜单加载
- **THEN** SinglePlayer 按钮 SHALL 使用 `bevy_ui_widgets::Button` 组件

#### Scenario: pause 按钮迁移

- **WHEN** 暂停菜单加载
- **THEN** Resume/Restart/Menu 按钮 SHALL 使用 `bevy_ui_widgets::Button` 组件

#### Scenario: gameover 按钮迁移

- **WHEN** 游戏结束界面加载
- **THEN** Restart/Menu 按钮 SHALL 使用 `bevy_ui_widgets::Button` 组件

### Requirement: 按钮点击使用 Activate Observer

按钮点击 SHALL 通过 `Activate` 事件 Observer 处理，替代 `Changed<Interaction>` 轮询。

#### Scenario: menu 按钮点击

- **WHEN** 玩家点击 SinglePlayer 按钮
- **THEN** `Activate` 事件 SHALL 触发，Observer 切换游戏状态到 `Playing`

#### Scenario: pause 按钮点击

- **WHEN** 玩家点击 Resume 按钮
- **THEN** `Activate` 事件 SHALL 触发，Observer 切换游戏状态到 `Playing`

#### Scenario: HUD 按钮点击

- **WHEN** 玩家点击兵种选择按钮
- **THEN** `Activate` 事件 SHALL 触发，Observer 设置城市产出兵种类型

### Requirement: 穿透保护使用 HoverMap

迁移完成后，`is_cursor_over_ui` SHALL 使用 `HoverMap` 检测光标是否在 UI 上，替代 `Interaction::Pressed` 查询。

#### Scenario: 穿透保护正常工作

- **WHEN** 玩家点击 UI 按钮
- **THEN** `is_cursor_over_ui` SHALL 通过 `HoverMap` 检测到光标在 UI 上，阻止游戏世界的点击处理

### Requirement: Phase 1a Observer 清理

Phase 1a 的验证 Observer（`observer.rs`）SHALL 被删除。

#### Scenario: observer.rs 删除

- **WHEN** Phase 2a 迁移完成
- **THEN** `crates/render_view/src/ui/observer.rs` 文件 SHALL 被删除
- **THEN** `ui/mod.rs` 中的 Observer 注册 SHALL 被移除

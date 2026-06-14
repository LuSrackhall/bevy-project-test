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

### Requirement: 穿透保护在混合期不中断

迁移期间，`is_any_ui_pressed` SHALL 同时检查 `PickingInteraction::Pressed` 和 `Pressed` 组件。

#### Scenario: 混合期穿透保护

- **WHEN** 玩家点击已迁移到 bevy_ui_widgets 的按钮
- **THEN** `is_any_ui_pressed` SHALL 通过 `Pressed` 组件检测到按钮被按下

#### Scenario: 混合期旧按钮穿透保护

- **WHEN** 玩家点击未迁移的旧版按钮
- **THEN** `is_any_ui_pressed` SHALL 通过 `PickingInteraction::Pressed` 检测到按钮被按下

### Requirement: Phase 1a Observer 清理

Phase 1a 的验证 Observer（`observer.rs`）SHALL 被删除。

#### Scenario: observer.rs 删除

- **WHEN** Phase 2a 迁移完成
- **THEN** `crates/render_view/src/ui/observer.rs` 文件 SHALL 被删除
- **THEN** `ui/mod.rs` 中的 Observer 注册 SHALL 被移除

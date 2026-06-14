## 1. Phase 1.5: PickingInteraction 统一迁移

- [ ] 1.1 将 `selection.rs` 中的 `is_any_ui_pressed` 从 `Query<&Interaction>` 迁移到 `Query<&PickingInteraction>`
- [ ] 1.2 将 `selection_click_system` 和 `command_issue_system` 的参数从 `Query<&Interaction>` 改为 `Query<&PickingInteraction>`
- [ ] 1.3 将 `hud.rs` 中 `update_bottom_panel` 的悬停检测从 `Interaction::Hovered` 改为 `PickingInteraction::Hovered`
- [ ] 1.4 删除所有 `Interaction` 相关的 import 和查询
- [ ] 1.5 编译验证：`cargo build` 无错误
- [ ] 1.6 运行验证：选中、穿透保护、悬停百科功能正常

## 2. Phase 2a: 简单按钮迁移（menu + pause + gameover）

- [ ] 2.1 在 `render_view/Cargo.toml` 启用 `experimental_bevy_ui_widgets` feature
- [ ] 2.2 将 `menu.rs` 的 SinglePlayer 按钮迁移到 `bevy_ui_widgets::Button` + `Activate` Observer
- [ ] 2.3 删除 `menu_button_system` 函数
- [ ] 2.4 将 `pause.rs` 的 3 个按钮迁移到 `bevy_ui_widgets::Button` + `Activate` Observer
- [ ] 2.5 删除 `pause_button_system` 函数
- [ ] 2.6 将 `gameover.rs` 的 2 个按钮迁移到 `bevy_ui_widgets::Button` + `Activate` Observer
- [ ] 2.7 删除 `gameover_button_system` 函数
- [ ] 2.8 更新 `is_any_ui_pressed` 同时检查 `PickingInteraction::Pressed` 和 `Pressed`
- [ ] 2.9 编译验证：`cargo build` 无错误
- [ ] 2.10 运行验证：所有按钮功能正常，穿透保护不中断

## 3. Phase 2b: HUD 按钮迁移

- [ ] 3.1 将 `soldier_type_button_system` 迁移到 `bevy_ui_widgets::Button` + `Activate` Observer
- [ ] 3.2 将 `toolbar_button_system` 迁移到 `bevy_ui_widgets::Button` + `Activate` Observer
- [ ] 3.3 将 `shield_button_visibility_system` 迁移（按钮可见性控制）
- [ ] 3.4 将命令卡按钮（移动/攻击/停止/驻守）迁移到 `bevy_ui_widgets::Button`
- [ ] 3.5 删除旧的 `Interaction` 轮询系统
- [ ] 3.6 编译验证
- [ ] 3.7 运行验证：HUD 按钮功能正常

## 4. Phase 2c: Seek Panel 迁移

- [ ] 4.1 将 `SeekIssueBtn` 迁移到 `bevy_ui_widgets::Button` + `Activate` Observer
- [ ] 4.2 将下拉菜单迁移到 `MenuPopup` + `MenuButton` + `MenuItem`
- [ ] 4.3 Observer 化范围输入框（`Pointer<Press>` 触发激活，保留手写数字逻辑）
- [ ] 4.4 从 `SeekPanelState` 移除 `dropdown_open` 字段
- [ ] 4.5 删除 `seek_panel_dropdown_system` 中的 `Interaction` 轮询
- [ ] 4.6 编译验证
- [ ] 4.7 运行验证：Seek Panel 功能正常

## 5. Phase 2d: 视觉反馈系统

- [ ] 5.1 定义 `ButtonTheme` 组件（normal/hovered/pressed 颜色）
- [ ] 5.2 实现 `button_style_system`（查询 `PickingInteraction` + `ButtonTheme`，更新 `BackgroundColor`）
- [ ] 5.3 为所有按钮添加默认 `ButtonTheme`
- [ ] 5.4 编译验证
- [ ] 5.5 运行验证：按钮悬停/按压有视觉反馈

## 6. Phase 2e: 清理

- [ ] 6.1 删除 `observer.rs`（Phase 1a 验证代码）
- [ ] 6.2 删除 `ui/mod.rs` 中的 Observer 注册
- [ ] 6.3 将 `is_any_ui_pressed` 简化为只检查 `Pressed` 组件
- [ ] 6.4 删除所有 `PickingInteraction` 相关代码
- [ ] 6.5 确认无 `Interaction` 残留引用
- [ ] 6.6 编译验证
- [ ] 6.7 运行验证：全量功能正常

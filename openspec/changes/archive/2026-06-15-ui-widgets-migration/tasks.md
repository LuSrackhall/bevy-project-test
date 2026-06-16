## 1. Phase 2a: 全量按钮迁移

- [x] 1.1 在 `render_view/Cargo.toml` 启用 `experimental_bevy_ui_widgets` feature
- [x] 1.2 迁移 `menu.rs`：SinglePlayer 按钮 → `bevy_ui_widgets::Button` + `Activate` Observer，删除 `menu_button_system`
- [x] 1.3 迁移 `pause.rs`：3 个按钮 → `bevy_ui_widgets::Button` + `Activate` Observer，删除 `pause_button_system`
- [x] 1.4 迁移 `gameover.rs`：2 个按钮 → `bevy_ui_widgets::Button` + `Activate` Observer，删除 `gameover_button_system`
- [x] 1.5 迁移 `hud.rs` 兵种按钮：`soldier_type_button_system` → Observer
- [x] 1.6 迁移 `hud.rs` 工具栏按钮：`toolbar_button_system` → Observer
- [x] 1.7 迁移 `hud.rs` 盾牌按钮可见性：`shield_button_visibility_system` 迁移
- [x] 1.8 迁移 `hud.rs` 命令卡按钮（移动/攻击/停止/驻守）→ `bevy_ui_widgets::Button`
- [x] 1.9 迁移 `hud.rs` Seek Panel 下发按钮：`seek_panel_issue_system` → Observer
- [x] 1.10 迁移 `hud.rs` Seek Panel 下拉菜单：→ `MenuPopup` + `MenuButton` + `MenuItem`（详见 seek-panel-menupopup-migration）
- [x] 1.11 迁移 `hud.rs` Seek Panel 范围输入框：Observer 化（保留手写数字逻辑）
- [x] 1.12 从 `SeekPanelState` 移除 `dropdown_open` 字段
- [x] 1.13 穿透保护改用 `HoverMap` + `is_cursor_over_ui`（替代原计划的 `Query<&Pressed>`）
- [x] 1.14 更新 `update_bottom_panel` 悬停检测：使用 `HoveredSoldierType` 资源 + `Pointer<Over>`/`Pointer<Out>` Observer
- [x] 1.15 删除 `observer.rs`（Phase 1a 验证代码）和 `ui/mod.rs` 中的 Observer 注册
- [x] 1.16 确认无 `Interaction` 残留引用
- [x] 1.17 编译验证：`cargo build` 无错误
- [x] 1.18 运行验证：所有按钮功能正常，穿透保护正常

## 2. Phase 2b: 视觉反馈系统

- [x] 2.1 定义 `ButtonTheme` 组件（normal/hovered/pressed 颜色）
- [x] 2.2 实现 `button_style_system`（查询 `Hovered` + `Pressed` + `ButtonTheme`，更新 `BackgroundColor`）
- [x] 2.3 为所有按钮添加默认 `ButtonTheme` 和 `Hovered::default()`
- [x] 2.4 编译验证
- [x] 2.5 运行验证：按钮悬停/按压有视觉反馈

## 3. Phase 2c: 清理与验证

- [x] 3.1 确认无 `Interaction` 残留引用
- [x] 3.2 确认无 `PickingInteraction` 残留引用
- [x] 3.3 编译验证
- [x] 3.4 运行验证：全量功能正常
- [x] 3.5 更新 `ui/CLAUDE.md` 文档（已完成）

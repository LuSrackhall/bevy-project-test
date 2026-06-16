## Why

当前 UI 层使用 Bevy 旧版 `bevy_ui::Button` + `Interaction` 组件轮询模式，与项目宪法（`ui/CLAUDE.md`）要求的"事件驱动交互模型"和"语义 UI 事件"不一致。`Interaction` 是 Bevy 旧 picking 模型的残留，每帧轮询 `Changed<Interaction>` 违反"React to semantic UI events instead of polling visual state"原则。`bevy_ui_widgets` 是 Bevy 官方推荐的新方向，提供 Observer 驱动的 `Activate` 事件和 `Pressed` 组件，与宪法目标一致。

Phase 1 已完成穿透修复（`Interaction::Pressed` 替代 `UiFocusBlocker`）和 Observer 机制验证（`Pointer<Press>` 可靠）。

## What Changes

- **Phase 2a**: 一次性迁移所有 25 个按钮到 `bevy_ui_widgets::Button` + `Activate` Observer，同步切换 `is_any_ui_pressed` 口径，删除 `observer.rs`
- **Phase 2b**: 新建集中式 `button_style_system` + `ButtonTheme` 组件，提供按钮视觉反馈
- **Phase 2c**: 清理所有 `Interaction` 残留引用，文档更新
- 启用 `experimental_bevy_ui_widgets` feature

## Capabilities

### New Capabilities

- `button-widgets-migration`: 将所有按钮从 `bevy_ui::Button` 迁移到 `bevy_ui_widgets::Button` + Observer
- `seek-panel-widgets-migration`: 将 Seek Panel 的下拉菜单迁移到 MenuPopup，输入框 Observer 化
- `button-visual-feedback`: 新建 ButtonTheme 组件和集中式 button_style_system

### Modified Capabilities

- `ui-click-detection`: `is_any_ui_pressed` 从 `Query<&Interaction>` 一次性迁移到 `Query<&Pressed>`

## Impact

- `crates/render_view/Cargo.toml`: 启用 `experimental_bevy_ui_widgets` feature
- `crates/render_view/src/selection.rs`: `is_any_ui_pressed` 迁移到 `Pressed`
- `crates/render_view/src/ui/hud.rs`: 所有按钮系统从 Interaction 轮询迁移到 Observer；新建 button_style_system
- `crates/render_view/src/ui/menu.rs`: 按钮迁移到 bevy_ui_widgets::Button
- `crates/render_view/src/ui/pause.rs`: 按钮迁移到 bevy_ui_widgets::Button
- `crates/render_view/src/ui/gameover.rs`: 按钮迁移到 bevy_ui_widgets::Button
- `crates/render_view/src/ui/observer.rs`: 删除（Phase 1a 验证代码）
- `crates/render_view/src/ui/mod.rs`: 注册新系统和 Observer
- 新增依赖: `bevy_ui_widgets`（通过 bevy feature flag）

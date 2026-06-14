## Why

点击 UI 元素时，游戏世界的选中状态被清除。根因是 `UiFocusBlocker` 手动机制没有覆盖所有 UI 按钮（`soldier_type_button_system`、`toolbar_button_system` 等系统未设置 `blocked = true`）。同时，右键点击 UI 时 `command_issue_system` 也缺乏穿透保护。

Bevy 0.18 的 Picking 系统已默认启用，UI 节点天然参与 picking 并阻止下层被 hover。当前代码通过 `Input<MouseButton>` + 手动 `UiFocusBlocker` 绕过了这套原生机制，导致穿透问题。应利用引擎原生能力替代手搓补丁。

## What Changes

- 用 Bevy Picking 系统的 `HoverMap` 资源替代 `UiFocusBlocker`，在 `selection_click_system` 和 `command_issue_system` 中检查光标是否在 UI 上
- 删除 `UiFocusBlocker` 资源、`reset_ui_focus_blocker` 系统及其所有引用
- 新增 Observer 模块，用 `Pointer<Click>` 事件验证 Observer 机制在当前项目中的可行性（为后续 `bevy_ui_widgets` 迁移做技术验证）
- 建立 `crates/render_view/src/ui/CLAUDE.md` 架构准则（已完成）

## Capabilities

### New Capabilities

- `ui-picking-click-detection`: 用 Picking 系统的 HoverMap 替代手动 UiFocusBlocker，实现 UI 点击穿透保护
- `observer-mechanism-validation`: 验证 Bevy Observer 机制在当前项目中的可行性，为后续 bevy_ui_widgets 迁移积累实证数据

### Modified Capabilities

（无现有 spec 需要修改）

## Impact

- `crates/render_view/src/selection.rs`: `selection_click_system` 和 `command_issue_system` 新增 HoverMap 检查
- `crates/render_view/src/ui/hud.rs`: 删除 `UiFocusBlocker` 定义、`reset_ui_focus_blocker` 系统、3 个 seek panel 系统中的 blocker 参数
- `crates/render_view/src/ui/mod.rs`: 删除 `UiFocusBlocker` 资源注册和 reset 系统注册，新增 observer 模块注册
- `crates/render_view/src/ui/observer.rs`: 新增文件，Observer 验证代码
- 无新增依赖（HoverMap 来自 bevy_picking，已随 bevy 默认包含）

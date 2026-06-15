## Why

当前 Seek Panel 的下拉菜单使用 `Display::None`/`Display::Flex` 切换可见性，但 Bevy 0.18 的 picking 系统在 `PreUpdate` 阶段读取 `ComputedNode.size`，而布局计算在 `PostUpdate` 阶段执行。当 `Display` 从 `None` 变为 `Flex` 时，picking 看到的仍然是旧的 `size == Vec2::ZERO`，导致 popup 内部按钮的 Pointer 事件永远不生成。

这是 Bevy 0.18 的已知时序问题，无法通过 `Pickable::IGNORE`、`Visibility::Hidden` 或手动 hit-test 解决。唯一从架构层面解决的方案是使用 `bevy_ui_widgets::MenuPopup`，它通过 spawn/despawn 管理 popup 生命周期，新实体诞生时就有正确的布局和 size。

## What Changes

- 注册 `MenuPlugin` + `PopoverPlugin` + `TabNavigationPlugin` + `InputDispatchPlugin`
- 将下拉菜单从 `Display::None/Flex` + `Pointer<Click>` observer 迁移到 `MenuPopup` + `MenuItem` + `Activate` observer
- 删除 `seek_panel_dropdown_system`（大部分逻辑由 MenuPopup 内建机制替代）
- 简化 `SeekPanelState`（删除 `dropdown_open`、`trigger_clicked`）
- 清理 `Pointer<Click>` observer 和 `Pickable::IGNORE` workaround

## Capabilities

### New Capabilities

- `seek-panel-menupopup`: 使用官方 MenuPopup 组件替代手写下拉菜单

### Modified Capabilities

- `seek-panel-widgets-migration`: 下拉菜单部分的实现方式从 Display 切换改为 spawn/despawn

## Impact

- `crates/render_view/src/lib.rs`: 注册新插件
- `crates/render_view/src/ui/hud.rs`: 重构下拉菜单 spawn 代码，删除 seek_panel_dropdown_system
- `crates/render_view/src/ui/mod.rs`: 删除 seek_panel_dropdown_system 注册
- 新增依赖: `MenuPlugin`、`PopoverPlugin`（通过 bevy_ui_widgets）

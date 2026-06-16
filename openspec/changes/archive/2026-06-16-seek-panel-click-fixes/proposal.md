## Why

两个 UX bug：(1) 点击输入框后光标不立即显示，用户不知道已进入输入状态；(2) 操作索敌面板时点击事件穿透到游戏世界，导致兵种选中状态丢失。

## What Changes

- **修改** `seek_panel_input_system`：点击激活时立即显示光标（`▌`）
- **修改** `selection_click_system` 的 UI 守卫：从检查全局 `Interaction` 改为检查点击位置是否在任何 UI 节点内

## Capabilities

### New Capabilities

- `seek-click-fix`: 输入框光标即时显示 + 点击穿透防护

### Modified Capabilities

（无）

## Impact

- **render_view/src/ui/hud.rs**: `seek_panel_input_system` 显示逻辑
- **render_view/src/selection.rs**: `selection_click_system` UI 守卫逻辑

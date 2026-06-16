## Why

`drag_select_system` 在框选过程中，当鼠标移出窗口时，`window.cursor_position()` 返回 `None`，系统直接 `return`，导致 `is_dragging` 保持 `true`、框选矩形卡在屏幕上。鼠标在窗口外松开时，`just_released` 可能不触发（基于窗口事件），框选永远无法完成。

## What Changes

- 当 `cursor_position()` 返回 `None` 且 `is_dragging` 为 `true` 时，保持 `drag_current` 不变（已在窗口边缘），继续执行后续逻辑
- 将 `mouse.just_released(MouseButton::Left)` 改为 `!mouse.pressed(MouseButton::Left)` 检测鼠标松开，确保窗口外松开也能触发

## Capabilities

### New Capabilities

- `drag-select-window-escape`: 框选过程中鼠标移出窗口时，保持框选状态并在松开时完成选中

### Modified Capabilities

（无）

## Impact

- `crates/render_view/src/selection.rs`: `drag_select_system` 函数修改

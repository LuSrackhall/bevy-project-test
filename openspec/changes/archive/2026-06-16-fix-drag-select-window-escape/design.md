## Context

`drag_select_system`（`selection.rs:141-197`）在框选过程中，当鼠标移出窗口时，`window.cursor_position()` 返回 `None`，系统在第 151 行 `return`，导致：
- `is_dragging` 保持 `true`
- `drag_current` 停留在最后的窗口内位置
- 框选矩形卡在屏幕上
- 鼠标在窗口外松开时，`just_released` 可能不触发，框选永远无法完成

## Goals / Non-Goals

**Goals:**
- 鼠标移出窗口时，保持 `drag_current` 不变（已在窗口边缘）
- 鼠标在窗口外松开时，完成框选
- 鼠标在窗口内松开时，行为不变

**Non-Goals:**
- 不处理多窗口场景
- 不改变框选的视觉样式
- 不改变 `selection_click_system`（单击选中）

## Decisions

### Decision 1: 分离"鼠标在窗口外"的提前返回

**选择**: 将 `cursor_position()` 为 `None` 的检查分为两种情况：
- 未在拖拽中 → 正常 `return`
- 正在拖拽中 → 跳过 `cursor_position` 检查，保持 `drag_current` 不变

**理由**: 区分"鼠标不在窗口内且未拖拽"和"鼠标不在窗口内但正在拖拽"两种语义。

### Decision 2: 改用 `!pressed` 检测松开

**选择**: 将 `mouse.just_released(MouseButton::Left)` 改为 `!mouse.pressed(MouseButton::Left)`。

**理由**: `pressed()` 基于全局状态，鼠标在窗口外松开时也能正确检测。`just_released` 基于窗口事件，跨窗口边界时不可靠。

## Risks / Trade-offs

**[钳制精度]** → 鼠标在窗口外时，`drag_current` 停在最后的窗口内位置。对于快速甩出窗口的场景，框选范围可能比预期小一帧。实际影响极小。

**[`!pressed` 误触发]** → 如果其他系统消费了 `just_released`，`!pressed` 可能在下一帧才检测到。延迟一帧完成框选，用户无感知。

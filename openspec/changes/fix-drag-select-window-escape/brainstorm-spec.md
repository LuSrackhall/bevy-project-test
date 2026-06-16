## Context

`drag_select_system`（`selection.rs`）在框选过程中，当鼠标移出窗口时，`window.cursor_position()` 返回 `None`，系统直接 `return`，导致 `is_dragging` 保持 `true`、`drag_current` 停留在最后位置，框选矩形卡在屏幕上。

Bevy 0.18 中，`ButtonInput<MouseButton>::just_released()` 基于窗口事件，鼠标在窗口外松开时可能不触发（取决于平台和窗口管理器）。

## Goals / Non-Goals

**Goals:**
- 鼠标移出窗口时，框选的 `drag_current` 钳制到窗口边缘
- 鼠标在窗口外松开时，完成框选（根据起点和边缘终点）
- 鼠标在窗口内松开时，行为不变

**Non-Goals:**
- 不处理多窗口场景
- 不改变框选的视觉样式
- 不改变 `selection_click_system`（单击选中）

## Decisions

### Decision 1: 鼠标在窗口外时保持 drag_current 不变

**选择**: 当 `cursor_position()` 返回 `None` 且 `is_dragging` 为 `true` 时，保持 `drag_current` 不变（已在窗口边缘附近），继续执行后续逻辑。

**备选方案**:
- A) 钳制到窗口边缘 → 需要计算边缘坐标，增加复杂度，且不知道鼠标越过了哪个边缘
- B) 取消框选 → 用户意图是选中，取消不符合预期
- C) 使用 `CursorLeft` 事件 → 增加复杂度，且不能处理"鼠标在窗口边缘反复进出"的情况

**理由**: 鼠标移出窗口时，`drag_current` 已经是窗口边缘附近的点。保持它不变，框选矩形自然停在边缘。这是最简单且效果足够好的方案。

### Decision 2: 改用 `!pressed` 检测松开

**选择**: 将 `mouse.just_released(MouseButton::Left)` 改为 `!mouse.pressed(MouseButton::Left)` 检测鼠标松开。

**备选方案**:
- A) 保持 `just_released` → 鼠标在窗口外松开时可能不触发
- B) 使用 `CursorLeft` + 手动状态管理 → 增加复杂度

**理由**: `pressed()` 基于全局状态，鼠标在窗口外松开时也能正确检测。`just_released` 基于窗口事件，跨窗口边界时不可靠。

### Decision 3: 分离"鼠标在窗口外"的提前返回

**选择**: 将 `cursor_position()` 为 `None` 的检查分为两种情况：
- 未在拖拽中 → 正常 `return`（不做任何事）
- 正在拖拽中 → 跳过 `cursor_position` 检查，保持 `drag_current` 不变

**理由**: 保持代码清晰，区分"鼠标不在窗口内且未拖拽"和"鼠标不在窗口内但正在拖拽"两种语义。

## Risks / Trade-offs

**[钳制精度]** → 鼠标在窗口外时，`drag_current` 停在最后的窗口内位置。对于快速甩出窗口的场景，框选范围可能比预期小一帧。实际影响极小。

**[`!pressed` 误触发]** → 如果其他系统消费了 `just_released`，`!pressed` 可能在下一帧才检测到。延迟一帧完成框选，用户无感知。

**[跨平台一致性]** → `cursor_position()` 在鼠标离开窗口时的行为可能因平台而异。`pressed()` 是全局状态，跨平台一致。

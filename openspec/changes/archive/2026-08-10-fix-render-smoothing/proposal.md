## Why

单位/城市/箭矢由 `draw_debug_shapes_system` 每帧直读 sim 世界 `LogicalPosition` 以 Gizmos 绘制,而 sim 以 20Hz 推进 → 图元位置 20Hz 跳变,单/多人普遍卡顿("卡顿现象根本没有任何改善"的根因)。presentation 层插值基础设施(`PresentationPlugin`/`PresentationPosition`/`InterpolationData`)从未注册、从未消费,是死代码。

## What Changes

- 新增 `PrevSimPositions(HashMap<sim Entity, Vec2>)` 资源(render_view)
- 新增 `capture_prev_positions_system`:每帧在 `SimulationTickSet` 前捕获 sim 各实体 `LogicalPosition`
- 修改 `draw_debug_shapes_system`:按 `TickClock` accumulator 计算 alpha,对城市/士兵/箭矢用 `lerp(prev, current, alpha)` 位置绘制,并回写 prev map
- 新实体(无 prev)回退到 current;静态图元(边界墙)不插值

## Capabilities

### New Capabilities
<!-- 无新能力 -->

### Modified Capabilities
- `render-view-crate`: DebugShape 几何体渲染要求增加"图元位置须按 TickClock 插值渲染"。

## Impact

- `crates/render_view/src/debug_shape.rs`(capture 系统 + draw 插值)
- `crates/render_view/src/lib.rs`(注册 capture 系统)
- 新增测试(单元级:无 tick 静止 / tick 后插值 / 新实体回退)
- 规格 `openspec/specs/render-view-crate/spec.md`(要求变更)

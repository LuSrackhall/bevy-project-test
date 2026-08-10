## Context

单位/城市/箭矢的渲染由 `draw_debug_shapes_system`(render_view/src/debug_shape.rs)实现:每帧从 sim 世界直接读取 `LogicalPosition` 并用 Gizmos 绘制。sim 以 20Hz(50ms/tick)推进,因此**图元位置以 20Hz 快照跳变**,在 60fps 渲染下产生明显卡顿——单人和多人模式一样受影响。这解释了用户"卡顿现象根本没有任何改善":历次变更从未触及渲染路径。

presentation 层已存在插值基础设施(`PresentationPlugin` / `compute_alpha_system` / `interpolate_positions_system` / `PresentationPosition` / `InterpolationData`),但:
- `PresentationPlugin` **从未被注册**到 app
- `InterpolationData` 仅在实体生成时赋值(`previous == current == 初始位置`),**从无按 tick 更新**
- `PresentationPosition` **从未被任何渲染系统消费**

整套插值基础设施是死代码。渲染直接读 sim 世界位置,绕过了它。

## Goals / Non-Goals

**Goals:**
- 图元位置在 60fps 渲染帧间平滑插值,消除 20Hz 跳变
- 覆盖 `draw_debug_shapes_system` 绘制的全部运动图元(士兵、箭矢;城市静态也走同一路径)
- 不依赖现存的死代码基建(presentation crate),方案自包含于 render_view
- 保持确定性(插值仅影响渲染,不影响 sim 世界状态)

**Non-Goals:**
- 不改 sim 层(不引入 PrevPosition 组件到 simulation)
- 不解决 Gizmos 每帧大量 circle_2d 的性能开销(独立性能问题)
- 不将 gizmo 渲染迁移为 sprite 渲染(重构过大)
- 不解决网络停滞导致的 driver tick 停顿(插值无法合成缺失数据)

## Decisions

**1. 双映射 + tick 边界交换(选定)** — 在 `draw_debug_shapes_system` 内部维护插值状态,不引入独立 capture 系统:

- 新增 `#[derive(Resource, Default)] struct RenderInterpolation { prev: HashMap<sim Entity, Vec2>, cur: HashMap<sim Entity, Vec2>, last_tick: u32 }`
- `draw_debug_shapes_system` 增加入参 `tick_clock: Res<TickClock>` 与 `interp: ResMut<RenderInterpolation>`;每帧:
  1. 若 `tick_clock.current_tick != interp.last_tick`(本帧完成了一个 tick):`std::mem::swap(&mut prev, &mut cur)`(旧 cur = 上一 tick 位置 成为新 prev),`cur.clear()`,`last_tick = current_tick`
  2. 遍历 sim 实体:`current = 当前 LogicalPosition`,`prev_pos = interp.prev.get(e).copied().unwrap_or(current)`,`alpha = (accumulator / tick_duration).clamp(0,1)`,`pos = prev_pos.lerp(current, alpha)`,以 `pos` 绘制;`interp.cur.insert(e, current)`

**时序正确性**(标准 1-tick-延迟插值):tick 完成帧 render `lerp(prev, current, alpha≈0)` = 上一 tick 位置(连续,无回跳);随 alpha 0→1 平滑滑向当前 tick 位置。新实体(无 prev)回退 current。

**2. 覆盖范围** — 城市、士兵、箭矢三处绘制均改用插值位置;边界墙等静态图元绘制位置不变。

**3. 调度** — `draw_debug_shapes_system` 已在 SimulationTickSet 之后运行(visual systems group 在 SyncEntitiesSet 后),天然拿到 tick 后位置,无需改调度。

## Risks / Trade-offs

- [双映射每帧 O(实体) 内存/查找] → 与 gizmo 绘制本身的查询同一量级,可忽略
- [多 tick 爆发(stall 后)] → tick 帧 alpha 可能 >0,从 prev 一次滑向最终位置,优于硬跳;网络停滞问题另案(非本变更)
- [destroyed 实体 stale 记录] → swap 后残留无害,从不被查询
- [渲染比 sim 延迟 1 tick(50ms)] → 标准做法,视觉无感;移动命令本就有 input_delay,可接受

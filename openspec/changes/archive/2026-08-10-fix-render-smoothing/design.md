## Context

sim 以 20Hz 推进,`draw_debug_shapes_system` 每帧直读 sim `LogicalPosition` 绘制图元 → 位置 20Hz 跳变,60fps 渲染下明显卡顿,单/多人通用。presentation 层插值基建是死代码(插件未注册、InterpolationData 不更新、PresentationPosition 无消费方)。

## Goals / Non-Goals

**Goals:**
- 运动图元位置按 TickClock 插值,消除 20Hz 跳变
- 自包含于 render_view,不依赖死代码基建
- 保持确定性(插值仅渲染,不改 sim)

**Non-Goals:**
- 不改 sim;不清理死代码基建;不做 gizmo→sprite 迁移;不解决网络停滞 tick 停顿

## Decisions

**1. RenderInterpolation 双映射资源** — `{ prev: HashMap<sim Entity, Vec2>, cur: HashMap<sim Entity, Vec2>, last_tick: u32 }`。

**2. tick 边界交换** — draw 每帧:若 `current_tick != last_tick`,`swap(prev, cur)` 使旧 cur(上一 tick 位置)成为新 prev,`cur.clear()`,更新 `last_tick`;随后遍历 sim 实体,`pos = lerp(prev.get(e) 或 current, current, alpha)`,绘制后 `cur.insert(e, current)`。

**3. alpha 语义** — `alpha = (accumulator / tick_duration).clamp(0,1)`。tick 完成帧 alpha≈0 → 渲染 prev(上一 tick 位置,连续);随后帧 alpha 0→1 平滑滑向当前。标准 1-tick-延迟插值。

**4. 覆盖** — 城市/士兵/箭矢均插值;边界墙静态图元不插值。

**5. 调度** — draw 已在 SimulationTickSet 之后运行(visual systems group 在 SyncEntitiesSet 后),天然拿到 tick 后位置,无需改调度。

**6. 测试(单元级)** — 构造 render_view 最小 app 注入 `SimulationWorld` + `TickClock` + `RenderInterpolation`,运行 draw(用空 Gizmos 不可行,故提取插值逻辑为可测纯函数 `interpolate_pos(prev, current, alpha)` 或直接对资源状态断言):
- 无 tick:位置不变(插值恒等)
- tick 边界交换:一次 tick 后 `prev` 携带旧位置、`cur` 携带新位置
- 中间 alpha:渲染位置在 prev 与 current 之间
- 新实体(无 prev)回退 current

## Risks / Trade-offs

- [双映射每帧 O(实体)] → 与 gizmo 绘制同量级,可忽略
- [stall 后多 tick 爆发] → tick 帧 alpha 可能 >0,从 prev 一次滑向最终位,优于硬跳;网络停滞另案
- [渲染 1 tick 延迟] → 标准做法,视觉无感,与 input_delay 叠加可接受

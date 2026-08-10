## 1. 渲染插值(render_view)

- [x] 1.1 新增 `RenderInterpolation { prev, cur, last_tick }` 资源(debug_shape.rs),`init_resource` 注册
- [x] 1.2 `draw_debug_shapes_system` 增加 `TickClock` + `RenderInterpolation` 入参;tick 边界 `swap(prev, cur)` + `cur.clear()` + 更新 `last_tick`
- [x] 1.3 城市/士兵/箭矢绘制位置改为 `lerp(prev.get(e) 或 current, current, alpha)` 并回写 `cur`
- [x] 1.4 边界墙等静态图元保持不插值

## 2. 测试(单元级)

- [x] 2.1 无 tick 时位置不变(插值恒等)
- [x] 2.2 tick 边界交换:一次 tick 后 prev 携带旧位置、cur 携带新位置
- [x] 2.3 中间 alpha 渲染位置位于 prev 与 current 之间
- [x] 2.4 新实体(无 prev)位置回退 current

## 3. 验证

- [ ] 3.1 `cargo test --workspace` 全绿

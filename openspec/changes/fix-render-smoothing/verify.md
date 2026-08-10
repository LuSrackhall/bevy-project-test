## Verification Summary

**Change:** fix-render-smoothing

| Dimension | Status |
|-----------|--------|
| Completeness | 8/8 tasks 完成 |
| Correctness | 插值时序正确(1-tick-延迟,边界零跳变),新局清空缓存 |
| Coherence | 与 design.md 一致;3 个评审 agent 通过 |

### Key Changes
- `crates/render_view/src/debug_shape.rs`: 新增 `RenderInterpolation{prev, cur, last_tick}` 双映射资源;`draw_debug_shapes_system` 城市/士兵/箭矢位置改为 `lerp(prev, current, alpha)` 插值绘制并回写 cur;8 个单元测试
- `crates/render_view/src/lib.rs`: 注册 RenderInterpolation;`reset_game_system` 新局清空插值缓存;`draw_debug_shapes_system` 显式 `.after(SimulationTickSet)`

### 评审结论
- **代码评审**: APPROVE(插值时序数学验证正确;新局残留 bug 已由 reset 清缓存解决)
- **宪法合规**: COMPLIANT(仅 render_view 层只读 sim,插值不写回仿真)
- **测试评审**: INADEQUATE → 已补强(新增 tick-then-glide 完整周期、alpha=0/1 边界、prev 区间稳定测试),现覆盖充分

### Issues
- 无 CRITICAL / WARNING(评审 WARNING 已处理:显式 `.after(SimulationTickSet)`)
- SUGGESTION(记录): seek 结束后存留实体从前 seek 位置滑 ~50ms——回放路径,非核心,可接受;presentation 层死代码基建留待后续清理(超范围)

### 测试
- `cargo test --workspace` 全绿(217 通过,0 失败)

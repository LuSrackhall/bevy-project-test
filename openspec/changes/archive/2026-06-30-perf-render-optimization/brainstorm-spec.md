# perf-render-optimization: 渲染层性能优化

## Context

Simulation 层经过两轮优化后已不再是瓶颈（1000 单位战斗 tick = 10.6ms，预算 50ms）。但用户反馈 1500+ 单位仍卡顿，瓶颈在渲染层。

渲染层每帧（60fps）运行两个高成本系统：

| 系统 | 每帧成本（1000 单位） | 问题 |
|------|---------------------|------|
| `unit_info_bar_system` | 11000 组件更新 + format! 分配 | 每帧对每个单位无条件更新 |
| `draw_debug_shapes_system` | 2000-6000 gizmo 调用 | 已 cfg-gated，可关闭 |

`unit_info_bar_system` 是唯一无条件运行的高成本系统：每个单位 8-11 个子实体（HP bar、EXP bar、shield bar、Text2d），每帧无条件调用 `format!` + 更新 Text2d，即使值未变化。

## Goals / Non-Goals

**Goals:**
- Phase 1：InfoBar 脏标记（缓存 HP/Level/EXP，值不变时跳过 format! + Text2d 更新）
- Phase 2：视口裁剪（Camera AABB 过滤屏幕外单位）
- Phase 3：默认 InfoBarMode=Selected（仅选中单位显示 bar）

**Non-Goals:**
- 替换 Gizmos 为 instanced rendering（100k+ 才需要）
- 修改 presentation 层
- 改变 simulation 层

## Decisions

### D1: 手动缓存而非 Changed<T>

**决策**：用 `Local<HashMap<UnitId, CachedState>>` 缓存 HP/Level/EXP，比较后跳过更新。

**理由**：InfoBar 通过 `NonSendMut<SimulationWorld>` 访问仿真世界，Health/Level 在仿真 World 中，不在 Bevy ECS World 中。Bevy 的 `Changed<T>` 只对 Bevy ECS 组件有效，无法使用。

**缓存结构**：
```rust
struct CachedState {
    hp_cur: u32,
    hp_max: u32,
    level: u32,
    exp: u64,
    shield_hp: u32,
    shield_max: u32,
}
```

### D2: 缓存清理复用 dead_ids 循环

**决策**：在现有的 `dead_ids` 清理循环（lines 257-266）中同步清理缓存。

**理由**：系统已通过 `bar_parts.keys()` 差集检测死亡单位。缓存清理可复用同一逻辑，无需额外事件订阅。

### D3: 视口裁剪插入点

**决策**：在 UnitBarInfo 收集后、per-unit 循环前（lines 254-268）插入 AABB 过滤。

**理由**：
- Camera.rs 已有 `OrthographicProjection.scale` 和 `Transform.translation`
- AABB = `center ± (window_size * scale / 2)`
- 每单位 4 次比较，~1μs/1000 单位
- 全部可见时跳过裁剪（`viewport_aabb.contains(map_bounds)` 短路）

### D4: 默认 InfoBarMode=Selected

**决策**：`InfoBarMode` 默认值从 `Classic` 改为 `Selected`。

**理由**：
- `Classic` 模式已对所有单位更新 bar（仅通过 should_show 控制可见性）
- `Selected` 模式仅对选中单位更新 → 成本从 O(N) 降到 O(selected)
- Ctrl+H 切换模式仍然可用

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 缓存值不一致（漏更新） | 无缓存条目 = dirty，无条件更新 |
| 视口裁剪在缩放时行为变化 | 全可见时跳过裁剪 |
| 默认 Selected 模式用户体验变化 | Ctrl+H 可切换回 Classic |

## Performance Estimates

| Phase | 1000 单位预期 | 说明 |
|-------|-------------|------|
| 当前 | ~16ms | 11000 ECS mutations + format! |
| +Phase 1 | 8-10ms | 90% format! 跳过 |
| +Phase 2 | 4-6ms | 仅更新屏幕内 300-400 单位 |
| +Phase 3 | <1ms | 仅 50 选中单位 |

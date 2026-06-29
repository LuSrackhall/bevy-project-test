## Context

渲染层 `unit_info_bar_system` 每帧对每个单位（1000）无条件更新 8-11 个子实体组件 + format! 分配。Simulation 层已优化到 10.6ms/tick，瓶颈转移至渲染。

## Goals / Non-Goals

**Goals:**
- 16ms → <6ms 渲染帧时间
- 1500+ 单位无帧率下降

**Non-Goals:**
- instanced rendering / shader-based bars
- 修改 simulation 层

## Decisions

### D1: 手动缓存（非 Changed<T>）

InfoBar 通过 NonSendMut<SimulationWorld> 访问仿真数据，Changed<T> 无效。用 Local<HashMap<UnitId, CachedState>> 手动比较。

### D2: 缓存清理复用 dead_ids

现有 dead_ids 差集循环（lines 257-266）已检测死亡单位。缓存清理复用同一逻辑。

### D3: 视口裁剪插入点

在 UnitBarInfo 收集后、per-unit 循环前（lines 254-268）插入 AABB 过滤。全可见时跳过。

### D4: 默认 InfoBarMode=Selected

仅选中单位显示 bar。Ctrl+H 可切换。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 缓存值不一致 | 无条目 = dirty，无条件更新 |
| Selected 模式用户体验 | Ctrl+H 切换回 Classic |

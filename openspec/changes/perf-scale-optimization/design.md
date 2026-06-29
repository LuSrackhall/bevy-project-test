## Context

当前 simulation crate 在 1000 单位下持续卡顿。代码审查发现三个结构性瓶颈。前两轮优化（UnitIdEntityIndex + SpatialHash for combat）已合并但效果不明显。

**实施结果**：性能从 1000 提升到 ~1300 单位。所有 Phase 1-6 完成，107 测试通过。瓶颈大概率在渲染层。

## Goals / Non-Goals

**Goals:**
- 修复 simulation 中的 O(n²) 算法缺陷（engagement Vec::find、overlap 重复构建）
- 消除 12+ 次冗余 HashMap 全表构建
- 为 SpatialHash 添加泛化 query_range 接口
- UnitIdEntityIndex 增量化
- 建立 profiling + benchmark 基础设施
- 补齐宪法合规缺失项

**Non-Goals:**
- 渲染层大规模重构
- 多线程并行
- SoA 组件布局
- 自定义 ECS 替换

## Decisions

### D1: engagement Vec::find → HashMap get()

`combat_engagement_system` 当前将 soldiers 收集到 `Vec<SoldierData>`，然后在 per-soldier 循环中用 `soldiers.iter().find(|(id, ..)| *id == suid)` 查找。外层循环 `sorted_soldier_uids` 保持排序，内层查找改为 HashMap `.get()` 不影响确定性——查找是 by key，不依赖迭代顺序。

实现：将 soldiers Vec 替换为 `HashMap<UnitId, (Entity, LogicalPosition, Faction, ...)>`。外层 `sorted_soldier_uids` 保持不变。

### D2: build_soldier_index 辅助函数

当前 12+ 处独立构建 `HashMap<UnitId, ...>` 的代码提取为辅助函数。每个系统独立调用（不共享 Resource），因为 melee_attack 可以杀死 entity，导致后续系统读到过期 Entity handles。

函数签名：
```rust
pub(crate) fn build_soldier_index(world: &mut World) -> HashMap<UnitId, SoldierSnapshot>
pub(crate) struct SoldierSnapshot {
    pub entity: Entity,
    pub pos: LogicalPosition,
    pub faction: Faction,
    pub health: Health,
    // ... 其他热路径字段
}
```

### D3: overlap_resolution SpatialHash 迭代间复用

`overlap_resolution_system` 当前在 max_iterations 循环内每次重建 SpatialHash。改为：首次构建后复用，仅在位置确实变化时重建。如果 overlap 解析收敛（无移动），跳过重建。

### D4: query_range 泛化接口

```rust
impl SpatialHash {
    pub fn query_range(&self, pos: FixedVec2, radius: i64) -> impl Iterator<Item = &SpatialEntry>
}
```

内部计算 `cells = (2 * ceil(radius / cell_size) + 1)^2`，遍历 BTreeMap 中对应的 cell range。各系统保持独立 cell_size。

### D5: UnitIdEntityIndex 增量化

当前每 tick 全量重建。改为：
- spawn 时 `index.insert(unit_id, entity)`
- despawn 时 `index.remove(unit_id)`
- 保留 `world.get_entity(entity).is_ok()` 作为安全网

实现方式：在 `consume_commands_system` 处理 spawn 命令时更新索引，在 despawn 处（melee_attack、attack_windup、city_interaction 等）更新索引。

### D6: Profiling 分层

```
simulation (零 profiling 依赖)
    ↓
bevy_adapter (tracing spans + tracy subscriber, feature-gated)
    ↓
render_view (debug_render feature gate)
```

tracing 仅在 bevy_adapter 层，围绕 `run_tick_default()` 调用添加 span。simulation 内部不加任何 instrumentation。

### D7: Benchmark 独立 crate

`crates/bench/` 作为独立 binary crate，依赖 simulation library。使用 golden_test.rs 模式构造 World。criterion benchmarks 覆盖完整 tick + 各 phase。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 辅助函数仍需每系统构建 HashMap（不如共享高效） | 正确性优先；O(N) 构建远小于 O(N²) bug |
| query_range archer 49 cells 查询成本 | 各系统保持独立 cell_size，archer 用 200 |
| 增量索引遗漏某些 despawn 路径 | 保留 Entity 存活检查作为安全网 |
| tracing 仅 tick 级别，无 phase 内部精度 | criterion bench 提供 phase 级精度 |

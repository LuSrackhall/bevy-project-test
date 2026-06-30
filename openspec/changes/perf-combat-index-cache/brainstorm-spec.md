# perf-combat-index-cache: 共享 SoldierIndex + SpatialHash

## Context

当前每 tick 4 个 combat 系统各自独立构建相似的数据结构，造成严重冗余：

| 系统 | 构建操作 | 扫描次数 |
|------|---------|---------|
| combat_engagement | all_units HashMap + faction_map + SpatialHash | 3 次 |
| melee_attack | build_soldier_index + faction_map + SpatialHash | 3 次 |
| archer_attack | build_soldier_pos_faction_map + soldier SpatialHash + city SpatialHash | 3 次 |
| arrow_movement | build_soldier_index + 3 HashMaps + soldier SpatialHash + city SpatialHash | 5 次 |

**合计每 tick：14 次全量 entity 扫描 + 6 次全量 SpatialHash 构建。**

Benchmark 数据：combat_engagement 在 3000 单位时 53ms（O(n*k), k≈n 在密集场景）。冗余扫描和构建贡献了 ~20% 的开销，剩下的 80% 在 per-unit 邻域查询。

目标：100,000 单位。需要将 O(n*k) 中的 k 控制在常数范围。

## Goals / Non-Goals

**Goals:**
- 所有 combat 系统共享一个 SoldierIndex per tick（14 次扫描 → 1 次）
- 所有 combat 系统共享一个 SpatialHash per tick（6 次构建 → 1 次）
- 按阵营索引 SpatialHash（k 减半，跳过友方单位）
- 为 100k 单位奠定基础

**Non-Goals:**
- SoA 数据布局（100k+ 可能需要，但当前架构已足够）
- 自定义 ECS
- 多线程并行

## Decisions

### D1: Tick-scoped Resource 缓存

**决策**：在 run_tick 开始时构建一次 `TickCombatIndex` Resource，包含 SoldierIndex + SpatialHash + FactionIndex。所有 combat 系统读取该 Resource，不再独立构建。

```rust
#[derive(Resource)]
struct TickCombatIndex {
    soldiers: HashMap<UnitId, SoldierSnapshot>,  // 替代 build_soldier_index
    soldier_spatial: SpatialHash,                 // cell_size=32
    faction_indices: HashMap<Faction, SpatialHash>, // 按阵营索引
}
```

**安全网**：find_entity_by_unit_id 已有 `world.get_entity(entity).is_ok()` 验证。即使索引中有已死亡 entity，验证返回 None。

**构建时机**：tick 开始时、combat_engagement 之前构建。movement 之后不重建——melee/archer/arrow 可以基于旧位置（误差在 1 tick 内，可接受）。

### D2: 按阵营索引 SpatialHash

**决策**：TickCombatIndex.faction_indices 按 Faction 分别构建 SpatialHash。查询时只扫敌方阵营的 cell。

**确定性**：每个阵营独立 BTreeMap + UnitId 排序，遍历顺序确定。

**效果**：k 减半（跳过 50% 友方单位）。

### D3: 不修改 SpatialHash 数据结构

**决策**：保持 BTreeMap + sorted Vec。不引入 HashMap。

**理由**：确定性要求（已多次评审确认）。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 共享索引中 Entity 过期 | find_entity_by_unit_id 有 Entity 存活验证 |
| build_soldier_index 字段不匹配 | SoldierSnapshot 已包含所有 combat 系统需要的字段 |
| 按阵营索引增加内存 | 每个阵营 ~100KB（1000 单位），可接受 |

## Performance Estimates

| 场景 | 当前 | 优化后 | 说明 |
|------|------|--------|------|
| tick/1000_combat | 10ms | 5-6ms | 消除冗余扫描 + k 减半 |
| tick/3000_combat | 42ms | 20-25ms | 仍超预算，但减半 |
| tick/100k_combat | ∞ | ? | 取决于 k 控制到多小 |

## Design Note

本方案专注于消除冗余（14 次扫描 → 1 次）。O(n*k) 中的 k 控制（如 cell_size 调优、query_range 替代 query_nearby）留待下一轮优化。

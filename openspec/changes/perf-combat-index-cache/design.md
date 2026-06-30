## Context

每 tick 4 个 combat 系统独立构建 14 次全量扫描 + 6 次 SpatialHash。冗余占 ~20% tick 时间。

## Goals / Non-Goals

**Goals:** 14→1 扫描，k 减半，100k 可行
**Non-Goals:** SoA 布局，多线程

## Decisions

### D1: TickCombatIndex Resource

```rust
#[derive(Resource)]
struct TickCombatIndex {
    soldiers: HashMap<UnitId, SoldierSnapshot>,
    soldier_spatial: SpatialHash,
    faction_indices: HashMap<Faction, SpatialHash>,
}
```

构建时机：tick 开始时（combat_engagement 之前）。各系统读取共享索引。

### D2: 按阵营索引

查询时只扫敌方阵营 SpatialHash，k 减半。

### D3: 安全性

`find_entity_by_unit_id` 的 `world.get_entity(entity).is_ok()` 验证保护 stale Entity。

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| archer/arrow 看到 phase 8 kill 的死单位 | find_entity_by_unit_id 返回 None，只是浪费工作 |
| soldier_movement 后位置偏移 | 1-tick 误差，lockstep 可接受 |

# perf-faction-spatial: 按阵营索引 + query_range

## Context

combat_engagement 等系统当前 query_nearby（3×3 cells）扫描所有阵营 unit，在热循环中逐条 faction check 跳过友方（50% 浪费）。TickCombatIndex 已有 `faction_spatial: HashMap<Faction, SpatialHash>`（cell_size=32），但未被使用。

## Goals / Non-Goals

**Goals:**
- combat 系统读取 `TickCombatIndex.faction_spatial[enemy_faction]`，跳过友方
- 用 `query_range(pos, radius)` 替代 `query_nearby(pos)`，确保 cell_size=32 覆盖足够
- 消除每个 combat 系统中的 faction_map 构建和 SpatialHash 构建

**Non-Goals:**
- 改变 SpatialHash 数据结构
- 多线程

## Decisions

### D1: faction_spatial + query_range

读取 `index.faction_spatial.get(&enemy_faction)`，用 `query_range(pos, seek_range)` 查询。跳过 100% 友方 unit，无需 faction check。

### D2: 安全处理空阵营

用 `.get()` 代替 `[]` 索引，空阵营时回退到空结果。

## Risks

| 风险 | 缓解 |
|------|------|
| cell_size=32 需扫更多 cell | query_range 自适应 sweep 数 |
| faction_spatial 不存在某阵营 | .get() 返回 None，回退到空 |
| faction_spatial 中包含 dead entity | 原有 find_entity_by_unit_id 验证
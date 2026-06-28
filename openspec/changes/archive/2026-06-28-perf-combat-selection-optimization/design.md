## Context

1000+ 单位卡顿。9 个卡顿点归类为 4 个根因：无空间索引、循环内排序、O(n) find_entity_by_unit_id、O(m*n) render_view 查询。已触发 Tier 2。

## Goals / Non-Goals

**Goals:** 消除 O(n²) 战斗热路径、find_entity_by_unit_id O(1)、selection/HUD 查询优化、保持确定性、修复 2 个 bug

**Non-Goals:** 不改游戏逻辑、不改宪法、不引入新 crate

## Decisions

### UnitIdEntityIndex（每 tick 重建）

simulation 内部派生数据，每 tick 开头从全量实体重建 HashMap<UnitId, Entity>。不违反 §17（派生数据无归属冲突）。O(n) 重建成本远低于被替换的 O(n²)。无需 ADR。

### SpatialHash BTreeMap + cell 内排序

HashMap→BTreeMap 保证 cell 遍历顺序确定。cell 内 Vec 按 UnitId 排序保证同一 cell 内遍历确定。SpatialHash 需扩展存储 UnitId。

### 4 个战斗系统推广 SpatialHash

combat_engagement、melee_attack、archer_attack、arrow_movement 从全量扫描改为 SpatialHash 查询。sorted_ids 排序从循环内移到循环外。

### 2 个 Bug 修复

city_interaction_system: UnitId(0) → 实际 UnitId。arrow_movement_system: 补发 UnitDestroyed 事件。

### render_view 优化

selection_visual_system + HUD 用 UnitIdEntityIndex O(1) 查找。

## Risks / Trade-offs

**[Risk] SpatialHash BTreeMap 比 HashMap 慢** → cell 数 < 1000 时差异可忽略

**[Risk] 40 个测试受影响** → 只改查询方式不改逻辑，行为不变

**[Risk] Bug 修复改变事件内容** → hash_world_state 不覆盖 SimulationEvents，不影响 golden_test

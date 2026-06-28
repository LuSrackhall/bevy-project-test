## Why

1000+ 单位场景下框选和索敌指令导致严重卡顿。根因是 4 个战斗系统使用 O(n²) 全量扫描无空间索引、`find_entity_by_unit_id` 为 O(n) 线性扫描、render_view 层查询为 O(m*n)。已触发宪法 Tier 2 条件（实体 > 1000），§4.2 禁止热点 O(n²)。

## What Changes

- 新增 `UnitIdEntityIndex(HashMap<UnitId, Entity>)` Resource，每 tick 重建，替代所有 `find_entity_by_unit_id` 线性扫描（9 处调用）
- SpatialHash 内部 HashMap→BTreeMap，cell 内 Vec 按 UnitId 排序，保证 §2.6 确定性
- combat_engagement_system 的 sorted_ids 排序从循环内移到循环外（性能改善）
- 修复 city_interaction_system 中 UnitDestroyed 事件 unit_id 硬编码为 UnitId(0) 的 bug
- 修复 arrow_movement_system 箭矢衰减销毁时未发出 UnitDestroyed 事件的 bug
- selection_visual_system 用 find_entity_by_unit_id O(1) 替代 O(m*n) 线性扫描
- find_entity_by_unit_id 返回前验证 Entity 存活状态，防止 despawn 后 panic
- HUD 多处 find_entity_by_unit_id 改用 simulation 的 O(1) 实现

## Capabilities

### New Capabilities
（无）

### Modified Capabilities
- `simulation-crate`: SpatialHash BTreeMap 改造、UnitIdEntityIndex、combat_engagement 排序优化、2 个 bug 修复

## Impact

- **性能**：selection 从 O(m*n) 降至 O(m)；find_entity_by_unit_id 从 O(n) 降至 O(1)；combat_engagement 排序从每士兵 O(u log u) 降至一次 O(u log u)；4 个战斗系统 SpatialHash 推广留到下一轮
- **确定性**：SpatialHash BTreeMap + cell 内排序保证等距敌人选择一致
- **Bug 修复**：city_interaction UnitId(0) + arrow_decay 无事件
- **测试影响**：103 测试全部通过
- **无新增依赖**：不引入新 crate

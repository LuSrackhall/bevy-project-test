## Why

1000+ 单位场景下框选和索敌指令导致严重卡顿。根因是 4 个战斗系统使用 O(n²) 全量扫描无空间索引、`find_entity_by_unit_id` 为 O(n) 线性扫描、render_view 层查询为 O(m*n)。已触发宪法 Tier 2 条件（实体 > 1000），§4.2 禁止热点 O(n²)。

## What Changes

- 新增 `UnitIdEntityIndex(HashMap<UnitId, Entity>)` Resource，每 tick 重建，替代所有 `find_entity_by_unit_id` 线性扫描（9 处调用）
- SpatialHash 内部 HashMap→BTreeMap，cell 内 Vec 按 UnitId 排序，保证 §2.6 确定性
- 4 个战斗系统推广 SpatialHash：combat_engagement_system、melee_attack_system、archer_attack_system、arrow_movement_system
- combat_engagement_system 的 sorted_ids 排序从循环内移到循环外
- 修复 city_interaction_system 中 UnitDestroyed 事件 unit_id 硬编码为 UnitId(0) 的 bug
- 修复 arrow_movement_system 箭矢衰减销毁时未发出 UnitDestroyed 事件的 bug
- selection_visual_system 用 UnitIdEntityIndex 替代 O(m*n) 线性扫描
- HUD 多处 find_entity_by_unit_id 改用 UnitIdEntityIndex O(1) 查找

## Capabilities

### New Capabilities
（无）

### Modified Capabilities
- `simulation-crate`: SpatialHash BTreeMap 改造、UnitIdEntityIndex、4 个战斗系统优化、2 个 bug 修复

## Impact

- **性能**：1000 单位时 combat 系统从 O(n²) 降至 O(n*k)，selection 从 O(m*n) 降至 O(m)
- **确定性**：SpatialHash BTreeMap + cell 内排序保证等距敌人选择一致
- **Bug 修复**：city_interaction UnitId(0) + arrow_decay 无事件
- **测试影响**：约 40 个 combat/soldier 测试需验证行为不变
- **无新增依赖**：不引入新 crate

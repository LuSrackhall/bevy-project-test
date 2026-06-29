## Why

4 个战斗系统（combat_engagement、melee_attack、archer_attack、arrow_movement）仍使用 O(n²) 全量扫描，是 1000+ 单位场景的主要性能瓶颈。上一轮已解决 find_entity_by_unit_id O(1)、selection_visual_system O(m)，但战斗系统的核心热路径未触及。

## What Changes

- combat_engagement_system 改用 SpatialHash（cell_size=64），预构建 faction_map
- melee_attack_system 改用 SpatialHash（cell_size=32），预构建 faction_map
- archer_attack_system 改用 SpatialHash（cell_size=200），预构建 faction_map
- arrow_movement_system 改用 SpatialHash（cell_size=32），预构建 soldier/city faction_map，处理穿透和城市碰撞

## Capabilities

### New Capabilities
（无）

### Modified Capabilities
- `simulation-crate`: 4 个战斗系统改用 SpatialHash 查询

## Impact

- **性能**：4 个系统从 O(n²) 降至 O(n*k)，k=邻域大小
- **确定性**：BTreeMap + cell 内 UnitId 排序保证一致
- **测试**：19 个现有测试无需调整

## MODIFIED Requirements

### Requirement: 战斗系统

simulation crate 的 combat_engagement_system、melee_attack_system、archer_attack_system、arrow_movement_system SHALL 使用空间索引（SpatialHash）进行邻近查询，SHALL NOT 对全部实体做全量扫描。每个系统 SHALL 使用独立的 SpatialHash 实例（不同 cell_size 适配不同查询范围）。SHALL 预构建 faction_map 避免在 SpatialHash 循环内访问 World。

#### Scenario: combat_engagement_system 使用 SpatialHash

- **WHEN** 1000 个士兵需要查找最近敌人
- **THEN** 系统通过 SpatialHash 查询局部邻域，复杂度 O(n*k)

#### Scenario: melee_attack_system 使用 SpatialHash

- **WHEN** 1000 个士兵近战查找范围内敌人
- **THEN** 系统通过 SpatialHash（cell_size=32）查询，复杂度 O(n*k)

#### Scenario: seek_range 超出 SpatialHash 覆盖时 fallback

- **WHEN** seek_range > 3 * cell_size
- **THEN** 系统 fallback 到全量扫描保证正确性

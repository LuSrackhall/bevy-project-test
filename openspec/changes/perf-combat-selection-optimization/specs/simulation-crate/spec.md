## MODIFIED Requirements

### Requirement: 战斗系统

simulation crate 的战斗系统 SHALL 使用空间索引（SpatialHash）进行邻近查询，SHALL NOT 对全部实体做全量扫描。SpatialHash SHALL 使用 BTreeMap 保证遍历顺序确定，cell 内 Vec SHALL 按 UnitId 排序。

#### Scenario: combat_engagement_system 使用空间索引

- **WHEN** 1000 个士兵需要查找最近敌人
- **THEN** 系统通过 SpatialHash 查询局部邻域，复杂度 O(n*k) 而非 O(n²)

#### Scenario: 等距敌人选择确定性

- **WHEN** 两个敌人与当前士兵距离相同
- **THEN** 选择结果由 BTreeMap cell 遍历顺序（字典序）+ cell 内 UnitId 排序决定，跨编译一致

## ADDED Requirements

### Requirement: UnitIdEntityIndex

simulation crate SHALL 在每 tick 开头重建 `UnitIdEntityIndex(HashMap<UnitId, Entity>)` Resource，从全量实体推导。SHALL 用于替代所有 `find_entity_by_unit_id` 线性扫描。

#### Scenario: O(1) 查找

- **WHEN** 调用 UnitIdEntityIndex 查找 UnitId 对应的 Entity
- **THEN** 复杂度 O(1)，不遍历全部实体

#### Scenario: 每 tick 重建

- **WHEN** run_tick 执行时
- **THEN** 在 Step 5 确定性仿真之前重建 UnitIdEntityIndex

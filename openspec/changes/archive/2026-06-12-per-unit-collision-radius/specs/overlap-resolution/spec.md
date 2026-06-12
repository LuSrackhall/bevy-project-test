## MODIFIED Requirements

### Requirement: 后处理重叠解算

每个 Tick 结束时 SHALL 运行 `overlap_resolution_system`。碰撞判定 SHALL 使用每兵种独立的 `collision_radius`（来自 `SoldierUnitConfig`），而非全局固定值。两兵最小距离 SHALL = `radius_A + radius_B`。重叠量 = 最小距离 - 实际距离，各推开 50%。

#### Scenario: 不同兵种碰撞体积不同

- **WHEN** 骑兵（radius=10）和弓兵（radius=5）距离为 12 单位
- **THEN** 最小距离 = 10+5 = 15，重叠 = 3，弓兵被推开 1.5，骑兵被推开 1.5

#### Scenario: 同兵种碰撞体积一致

- **WHEN** 两个步兵（radius=7）距离为 10 单位
- **THEN** 最小距离 = 7+7 = 14，重叠 = 4，各推开 2

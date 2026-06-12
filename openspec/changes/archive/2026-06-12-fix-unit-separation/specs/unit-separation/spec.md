## MODIFIED Requirements

### Requirement: 士兵空间分离

士兵在每 Tick 移动后 SHALL 与其他士兵保持最小间距（`separation_radius`，默认 8 内部单位）。分离计算 SHALL 使用两遍流程：第一遍收集所有士兵的 raw_pos（移动目标位置），第二遍用 raw_pos 构建 SpatialHash 后做邻居查询和分离力调整。城池产兵时 SHALL 在 spawn_pos 上施加确定性偏移（±8 单位），从源头分散新兵。

#### Scenario: 两个士兵汇聚时自动散开

- **WHEN** 两个士兵的目标位置距离 < 8 单位
- **THEN** 每 Tick 各自被推开（`separation_weight = 0.6`），最终自然分散到间距 ≥ 8

#### Scenario: 城池产兵不重叠

- **WHEN** 城池在一次产兵中连续生成多个士兵
- **THEN** 每个士兵的 spawn_pos 有不同的确定性偏移，不全部重叠在同一坐标

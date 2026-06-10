## ADDED Requirements

### Requirement: 士兵空间分离

士兵在每 Tick 移动后 SHALL 与其他士兵保持最小间距（`separation_radius`，默认 8 内部单位）。若目标位置与其他士兵距离 < `separation_radius`，SHALL 施加分离力推开。分离力仅作用于同一 Tick 的目标位置调整，SHALL NOT 改变士兵的逻辑移动速度。

#### Scenario: 两个士兵汇聚时自动散开

- **WHEN** 两个士兵的目标位置距离 < 8 单位
- **THEN** 每 Tick 各自被推开 small amount（`separation_weight = 0.4`），最终自然分散到间距 ≥ 8

#### Scenario: 士兵高速汇聚时不被推开过远

- **WHEN** 大量士兵（≥20）快速向同一目标汇聚
- **THEN** 分离力使士兵形成自然聚集而非重叠，但不会阻止整体移动方向

### Requirement: 空间哈希邻居查询

系统 SHALL 使用空间哈希网格进行 O(1) 邻居查询。网格 cell_size SHALL = `separation_radius × 2`（默认 16 单位）。每次查询 SHALL 检查当前格子 + 8 个邻接格子，共 9 格。

#### Scenario: 邻居查询仅返回附近士兵

- **WHEN** 查询某士兵位置周围 `separation_radius` 范围内的其他士兵
- **THEN** 返回结果仅包含距离 < `separation_radius` 的士兵，不包含远距离士兵

#### Scenario: 万人场景性能可接受

- **WHEN** 地图上有 10,000 个士兵
- **THEN** 构建网格 + 全部查询的计算量 < 200K 次距离比较

## ADDED Requirements

### Requirement: 后处理重叠解算

每个 Tick 结束时 SHALL 运行 `overlap_resolution_system`。该系统 SHALL 扫描全部 `SoldierMarker` 实体，对每对距离 < `min_separation`（默认 8 单位）的士兵施加推开力：沿连线方向各推开 50% 重叠量。系统 SHALL 迭代最多 `max_iterations`（默认 3）次，直到无重叠或达到上限。

#### Scenario: 移动后重叠被自动纠正

- **WHEN** 10 个兵移动到同一 waypoint 后堆叠在一起
- **THEN** overlap_resolution_system 将它们推开到间距 ≥ 8 单位

#### Scenario: 产兵重叠被纠正

- **WHEN** 城池产兵产生重叠（即使有产兵抖动）
- **THEN** 重叠在 Tick 结束前被解算系统推开

#### Scenario: 迭代提前退出

- **WHEN** 第一次迭代后已无重叠
- **THEN** 系统不执行后续迭代，直接退出

### Requirement: 重叠解算与逻辑隔离

overlap_resolution_system SHALL 仅依赖 `LogicalPosition` 组件。SHALL NOT 读取或修改 `Movement`、`Target`、状态机或任何战斗相关组件。SHALL 对所有来源的重叠一视同仁。

#### Scenario: 不依赖移动状态

- **WHEN** 两个兵以任何方式重叠（移动、产兵、被技能推送）
- **THEN** 均被推开，无论各自的 Movement 或 SoldierState 为何

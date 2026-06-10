## MODIFIED Requirements

### Requirement: 士兵组件与系统

simulation crate SHALL 定义士兵相关组件：`LogicalPosition`、`Movement`、`Health`、`Attack`、`FactionComponent`、`SoldierTypeComponent`、`Level`、`ShieldComponent`、`CityOrigin`、`SlowDebuff`。SHALL 提供 `soldier_movement_system` 基于 `Movement.target`（UnitId 或 waypoint 位置）和 `Movement.speed` 更新 `LogicalPosition`。SHALL 对 waypoint 和城池目标应用基于 UnitId 的确定性目标偏移（formation offset），防止多士兵汇聚时重叠。移动目标位置 SHALL 经过分离力调整，确保士兵间保持最小间距。

#### Scenario: 士兵向目标移动

- **WHEN** 士兵的 `Movement.target = Some(waypoint_id)`，且 waypoint 的 `LogicalPosition` 距离士兵位置 100 像素（Fixed 单位）
- **THEN** 执行一次 Tick 后，士兵的 `LogicalPosition` 向目标方向移动了 `speed × tick_duration` 距离

#### Scenario: 士兵到达目标

- **WHEN** 士兵的 `LogicalPosition` 和目标位置的平方距离 < 阈值平方（如 Fixed::from_int(5)²）
- **THEN** `Movement.target` 被清除为 `None`，士兵停止移动

#### Scenario: 骑兵不受战斗目标覆盖

- **WHEN** 骑兵处于 `Fighting` 状态且 `Movement.command_target` 不为 `None`
- **THEN** 骑兵继续向 `Movement.command_target` 移动，不因战斗状态而停在原地攻击

#### Scenario: 士兵间保持间距

- **WHEN** 两个士兵汇聚到间距 < `separation_radius`（默认 8 单位）
- **THEN** 每 Tick 双方目标位置被分离力推开，最终保持 ≥ `separation_radius` 的间距

#### Scenario: 多士兵 waypoint 移动不堆叠

- **WHEN** 多个士兵被命令移动到同一 waypoint
- **THEN** 每兵的目标位置 = waypoint + personal_offset(unit_id)，到达后自动形成方阵

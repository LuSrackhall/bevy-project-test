## MODIFIED Requirements

### Requirement: Tick 调度驱动

`tick_driver_system` 和 `replay_tick_driver_system` SHALL 被删除，替换为统一的 `simulation_driver_system`。`GameMode` 枚举和 `ReplayController` 资源 SHALL 被删除，替换为 `SimulationDriver` 资源。

#### Scenario: 统一驱动
- **WHEN** 应用运行
- **THEN** 只有 simulation_driver_system 驱动 tick 推进，无其他 tick 驱动系统

### Requirement: Tick 命令快照

录制逻辑 SHALL 移入 `simulation_driver_system`。在获取命令后、注入 simulation 前，如 Live + 录制开启 + 非 seek，SHALL 将命令副本保存到 ReplayRecorder。

#### Scenario: 录制在统一驱动中
- **WHEN** Live 模式、录制开启、非 seek
- **THEN** simulation_driver_system 将命令副本保存到 ReplayRecorder

## ADDED Requirements

### Requirement: GameActive 门控

simulation_driver_system 和 sync_entities_system SHALL 通过 `run_if(resource_exists_and_equals(GameActive(true)))` 门控。不再需要 GameMode 的 run_if 条件。

#### Scenario: 唯一外部门控
- **WHEN** GameActive = true
- **THEN** simulation_driver_system 运行（Live 和 Replay 都通过同一系统）

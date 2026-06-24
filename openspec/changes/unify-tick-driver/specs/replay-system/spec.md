## MODIFIED Requirements

### Requirement: Replay 回放

回放 SHALL 通过 SimulationDriver（CommandSource::Replay）驱动，不再使用独立的 replay_tick_driver_system。ReplayController 资源 SHALL 被删除，其功能合并到 SimulationDriver 和 SchedulerState。

#### Scenario: 回放走统一驱动
- **WHEN** Replay 模式激活
- **THEN** simulation_driver_system 通过 CommandSource::Replay 获取命令并驱动 tick

### Requirement: ReplayStatus 资源

ReplayStatus.is_replay SHALL 为展示态缓存（派生自 source），非权威状态。

#### Scenario: 派生状态
- **WHEN** SimulationDriver.source 为 Replay
- **THEN** ReplayStatus.is_replay = true

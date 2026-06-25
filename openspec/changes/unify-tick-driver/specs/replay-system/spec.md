## MODIFIED Requirements

### Requirement: Replay 回放

回放 SHALL 通过 SimulationDriver（CommandSource::Replay）驱动，不再使用独立的 replay_tick_driver_system。ReplayController 资源 SHALL 被删除，功能合并到 SimulationDriver 和 SchedulerState。

#### Scenario: 回放走统一驱动
- **WHEN** Replay 模式激活
- **THEN** simulation_driver_system 通过 CommandSource::Replay 获取命令并驱动 tick

### Requirement: ReplayStatus 资源

ReplayStatus.is_replay SHALL 为展示态缓存（派生自 SimulationDriver.source），非权威状态。

### Requirement: Replay 确定性

Replay 的命令录制和回放 SHALL 使用同一套 simulation_driver_system。simulation 层的位置查询 SHALL 使用 HashMap + 排序遍历，消除非确定性迭代。

#### Scenario: 录制回放一致性
- **WHEN** 使用新代码录制一局游戏并回放
- **THEN** 回放中的 AI 行为与原始对局完全一致

### Requirement: Tick 驱动兼容

TickClock SHALL 作为独立 Resource 存在（presentation 层读取），由 SimulationDriver 每帧同步。旧的 ReplayController 和 GameMode 资源 SHALL 被删除。

#### Scenario: 清理旧状态
- **WHEN** 回放结束或退出 Playing 状态
- **THEN** SimulationDriver 重置为 Live 模式，GameMode 重置为 Live，ReplayStatus 重置

## Why

当前 bevy_adapter 中有两个独立的 tick 驱动系统（`tick_driver_system` 和 `replay_tick_driver_system`），它们的命令注入和 tick 推进逻辑各自实现。快放/seek 后 AI 行为可能与原始对局不一致，因为两个系统存在微妙的时序差异。项目宪法 2.4 要求「同一套命令注入与消费流水线」，当前架构违反了这一原则。此问题同时影响 Replay 回放和未来 Lockstep 的断线重连（本质也是快放）。

## What Changes

- **BREAKING**: 删除 `tick_driver_system` 和 `replay_tick_driver_system`，替换为统一的 `simulation_driver_system`
- **BREAKING**: 删除 `GameMode` 枚举和 `ReplayController` 资源，替换为 `SimulationDriver` 资源
- 新增 `CommandSource` 枚举（Live/Replay），封装命令来源差异
- 新增 `SimulationDriver` 资源：三层分离（TickClock + SchedulerState + CommandSource）
- 新增 `DriverContext` 结构，传递运行时依赖（Bevy CommandBuffer 引用）
- 新增 `SimulationDriver::is_replay()` 辅助方法
- render_view 的 UI 控制改为修改 `SimulationDriver.scheduler`
- 统一 seek 逻辑到 `simulation_driver_system` 内部

## Capabilities

### New Capabilities
- `simulation-driver`: 统一的仿真驱动架构，包含 SimulationDriver、CommandSource enum、TickClock、SchedulerState、DriverContext，以及确定性保证的不变量

### Modified Capabilities
- `bevy-adapter-crate`: 删除 GameMode/ReplayController/tick_driver_system/replay_tick_driver_system，替换为 SimulationDriver + simulation_driver_system
- `game-lifecycle`: GameActive 作为唯一外部门控，不再需要 GameMode 的 run_if 条件
- `replay-system`: Replay 的录制/回放/seek 逻辑迁移到 SimulationDriver 架构下

## Impact

- **bevy_adapter 层重构**: 删除约 200 行分散的 tick 驱动代码，替换为约 300 行统一驱动代码
- **render_view 适配**: UI 控制逻辑从操作 GameMode/ReplayController 改为操作 SimulationDriver.scheduler
- **测试新增**: Driver 层确定性测试（不同调度密度、seek、命令单次消费）
- **未来收益**: Lockstep 网络只需实现新的 CommandSource 变体，不需要新建 tick 驱动系统

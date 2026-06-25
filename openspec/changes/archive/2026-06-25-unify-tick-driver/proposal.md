## Why

当前 bevy_adapter 中有两个独立的 tick 驱动系统（`tick_driver_system` 和 `replay_tick_driver_system`），它们的命令注入和 tick 推进逻辑各自实现。快放/seek 后 AI 行为可能与原始对局不一致，因为两个系统存在微妙的时序差异。项目宪法 2.4 要求「同一套命令注入与消费流水线」，当前架构违反了这一原则。此问题同时影响 Replay 回放和未来 Lockstep 的断线重连（本质也是快放）。

## What Changes

- **BREAKING**: 删除 `tick_driver_system` 和 `replay_tick_driver_system`，替换为统一的 `simulation_driver_system`
- **BREAKING**: 删除 `ReplayController` 资源，功能合并到 `SimulationDriver`
- 新增 `SimulationDriver` 资源：三层分离（TickClock + SchedulerState + CommandSource）
- 新增 `CommandSource` 枚举（Live/Replay），封装命令来源差异
- 新增 `DriverContext` 结构，传递运行时依赖
- 新增 `GameMode` 枚举（Live/Replay）作为输入系统门控（防止回放时输入系统干扰仿真）
- 新增 `world_fingerprint` 工具函数用于确定性调试
- 保留 `TickClock` 作为独立 Resource（presentation 层兼容），由 SimulationDriver 同步
- render_view 输入系统（command_issue 等）在 GameMode::Replay 时不运行
- simulation 层 HashMap 改为 HashMap + 排序遍历，兼顾 O(1) 查找与确定性

## Capabilities

### New Capabilities
- `simulation-driver`: 统一的仿真驱动架构，包含 SimulationDriver、CommandSource enum、TickClock、SchedulerState、DriverContext、GameMode 门控

### Modified Capabilities
- `bevy-adapter-crate`: 删除旧 tick 驱动系统和 ReplayController，替换为 SimulationDriver + simulation_driver_system + GameMode 门控
- `game-lifecycle`: GameActive + GameMode 双重门控，输入系统仅在 Live 模式运行
- `replay-system`: Replay 的录制/回放/seek 迁移到 SimulationDriver 架构下
- `simulation-crate`: HashMap → HashMap+排序遍历，消除非确定性迭代

## Impact

- **bevy_adapter 层重构**: 删除约 200 行分散的 tick 驱动代码，替换为约 350 行统一驱动代码
- **simulation 层修复**: 消除 3 处 HashMap 非确定性迭代（combat/mod.rs、soldier/mod.rs）
- **render_view 适配**: UI 控制逻辑迁移，输入系统增加 GameMode 门控
- **测试新增**: 4 个 Driver 层确定性测试 + 现有 93 个 simulation 测试通过
- **未来收益**: Lockstep 网络只需实现新的 CommandSource 变体

## Why

仿真层架构从第一天就为 Lockstep 和 Replay 设计（定点数、CommandBuffer 驱动、纯函数 run_tick），但实现层面存在 12-15 处 f32 运算直接参与仿真逻辑（概率判定、伤害计算、移动速度等），破坏了确定性保证。在引入 Replay/联机之前，必须先消除这些泄漏并通过黄金测试验证确定性。Replay 系统既是确定性的端到端验证工具，也是未来 P2P Lockstep 的前置基础设施。

## What Changes

- **BREAKING**: 所有仿真层概率/比例配置从 `f32`（如 `0.3`）改为 `u32` 万分比整数（如 `3000`），涉及 combat、soldier、city、ai 配置和逻辑
- **BREAKING**: `gen_probability()` 返回值从 `f32` 改为 `u32`（0-10000）
- 新增 simulation 层 `ReplayFile` 数据结构（seed + map_size + 命令序列）
- 新增 simulation 层 `SimulationSeed` 资源，持久化 RNG 种子
- 新增 simulation 层命令类型（GameCommand、Action 等）的 serde Serialize/Deserialize 支持
- 新增 bevy_adapter 层 `GameMode` 枚举（Live/Replay）、`ReplayRecorder`、`ReplayController`
- 新增 bevy_adapter 层 `replay_tick_driver_system`，与实时 tick 驱动互斥
- 新增 render_view 层 Replay 播放器 UI（播放/暂停、快进、进度条 seek）
- 新增 render_view 层主菜单 "Load Replay" 按钮和录制开关设置

## Capabilities

### New Capabilities
- `deterministic-simulation`: 消除 simulation 层所有 f32 仿真运算，概率用万分比整数，比例用整数乘除，确保单平台位精确确定性
- `golden-determinism-test`: 固定 seed + 固定指令序列的黄金测试，断言世界状态一致，CI 持续验证
- `replay-system`: 完整 Replay 系统——录制（每 tick 收集外部命令）、回放（从 seed 重建世界并注入命令）、播放器控制（暂停/快进/进度条 seek）

### Modified Capabilities
- `simulation-crate`: 新增 serde derives（GameCommand/Action/Fixed 等）、replay 模块（ReplayFile）、SimulationSeed 资源、gen_probability 改为万分比
- `bevy-adapter-crate`: 新增 GameMode 枚举、ReplayRecorder、ReplayController、replay_tick_driver_system
- `game-lifecycle`: 新增 Replay 游戏状态和从主菜单加载 Replay 的流程

## Impact

- **配置文件变更**: content/*.ron 中所有 f32 概率/比率字段改为 u32 万分比（破坏性变更，需全量更新配置）
- **仿真逻辑变更**: combat/mod.rs、soldier/mod.rs、ai/mod.rs 中约 12-15 处 f32 运算改为整数运算
- **新增依赖**: simulation crate 新增 serde Serialize 能力（已有 serde 依赖，仅补全方向）
- **测试影响**: 现有 combat 测试需要更新配置值（万分比），新增黄金确定性测试
- **风险**: f32→整数转换可能微妙影响游戏平衡（万分比精度 0.01% 足够覆盖所有配置）
- **架构债务**: render_view 直接写入仿真组件的问题单独追踪，不在本次修复范围

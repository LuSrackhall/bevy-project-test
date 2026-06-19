## MODIFIED Requirements

### Requirement: Tick 调度驱动

TickClock SHALL 保持现有实时对局的 tick 调度逻辑不变。回放模式 SHALL NOT 修改 TickClock，而是使用独立的 ReplayController 驱动。

#### Scenario: 实时模式不受影响
- **WHEN** GameMode 为 Live
- **THEN** TickClock 行为与现有实现完全一致

### Requirement: Tick 命令快照

在 tick_driver_system 中提取当前 tick 命令后、注入 simulation 前，SHALL 将命令副本保存到 ReplayRecorder（如录制已启用）。AI 在 run_tick 内部产生的命令 SHALL NOT 被录制。

#### Scenario: 录制拦截点正确
- **WHEN** Bevy 侧 CommandBuffer 包含 tick 42 的 3 条玩家命令
- **AND** 录制已启用
- **THEN** ReplayRecorder 记录 tick 42 的这 3 条命令

## ADDED Requirements

### Requirement: GameMode 枚举

bevy_adapter SHALL 定义 `GameMode` 枚举（Live/Replay），作为 Bevy Resource。tick_driver_system 和 replay_tick_driver_system 通过 `run_if` 条件基于 GameMode 互斥运行。

#### Scenario: 模式切换
- **WHEN** GameMode 从 Live 切换为 Replay
- **THEN** tick_driver_system 停止运行，replay_tick_driver_system 开始运行

### Requirement: ReplayRecorder 资源

bevy_adapter SHALL 定义 `ReplayRecorder` 资源，包含录制缓冲区（`Vec<(u32, Vec<GameCommand>)>`）、seed、map_size 和 is_recording 标志。

#### Scenario: 录制启停
- **WHEN** is_recording = true 且游戏进行中
- **THEN** 每 tick 的外部命令被追加到缓冲区
- **WHEN** is_recording = false
- **THEN** 命令不被记录

### Requirement: ReplayController 资源

bevy_adapter SHALL 定义 `ReplayController` 资源，包含 replay_data、current_tick、target_tick、speed、is_paused、is_seeking。

#### Scenario: 快进执行
- **WHEN** speed 设为 Fast4x 且游戏未暂停
- **THEN** 每帧执行 4 个 tick（调用 run_tick 4 次）

### Requirement: ReplayStatus 资源

bevy_adapter SHALL 暴露 `ReplayStatus { is_replay: bool, total_ticks: u32 }` 资源。render_view 读取此资源显示进度条，SHALL NOT import simulation 类型。

#### Scenario: 状态暴露
- **WHEN** GameMode 为 Replay，ReplayFile.total_ticks = 12000
- **THEN** ReplayStatus 返回 is_replay=true, total_ticks=12000

### Requirement: SimulationSeed 资源

bevy_adapter SHALL 在 init_simulation_world 时同时插入 SimulationSeed 资源。录制时从该资源读取 seed 写入 ReplayFile。

#### Scenario: Seed 可读取
- **WHEN** 使用 seed 42 初始化 World
- **THEN** SimulationSeed(42) 存在于 World 中

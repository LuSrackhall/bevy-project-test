## ADDED Requirements

### Requirement: SimulationDriver 资源

bevy_adapter SHALL 定义 `SimulationDriver` 资源，包含 `TickClock`（时序）、`SchedulerState`（调度）、`CommandSource`（命令来源）三层。`current_tick` 只在 `TickClock` 中持有，是唯一权威值。SimulationDriver 通过 `insert_resource(SimulationDriver::new_live())` 注册。

#### Scenario: 初始化为 Live 模式
- **WHEN** 应用启动
- **THEN** SimulationDriver 初始化为 Live 模式，speed_multiplier = 1，is_paused = false

#### Scenario: current_tick 唯一权威
- **WHEN** 任意系统读取当前 tick
- **THEN** 只能从 SimulationDriver.clock.current_tick 读取

### Requirement: CommandSource 枚举

bevy_adapter SHALL 定义 `CommandSource` 枚举（Live/Replay），通过 `commands_for_tick(tick, ctx)` 方法获取命令。

#### Scenario: Live 模式命令获取
- **WHEN** SimulationDriver.source 为 Live
- **THEN** 通过 DriverContext.bevy_cmds 过滤当前 tick 的命令

#### Scenario: Replay 模式命令获取
- **WHEN** SimulationDriver.source 为 Replay
- **THEN** 从 ReplayFile 读取当前 tick 的命令

### Requirement: simulation_driver_system

bevy_adapter SHALL 实现 `simulation_driver_system`，替代原 `tick_driver_system` 和 `replay_tick_driver_system`。系统 SHALL 按以下顺序执行：获取命令 → 注入 simulation CommandBuffer → 调用 run_tick。

#### Scenario: 统一流水线
- **WHEN** Live 模式或 Replay 模式
- **THEN** 每个 tick 的执行路径完全相同

#### Scenario: 系统调度顺序
- **WHEN** GameActive 为 true
- **THEN** simulation_driver_system.before(sync_entities_system)

### Requirement: TickClock 同步

TickClock SHALL 作为独立 Resource 注册（presentation 层兼容），由 `simulation_driver_system` 每帧同步 `current_tick` 和 `accumulator`。

#### Scenario: 双份同步
- **WHEN** simulation_driver_system 执行 tick
- **THEN** TickClock.current_tick 和 TickClock.accumulator 与 SimulationDriver.clock 同步

### Requirement: SchedulerState 调度控制

`SchedulerState` SHALL 包含 is_paused、speed_multiplier、seek_target、async_seek 字段。speed_multiplier 只影响每帧 tick 调度密度。

#### Scenario: 倍速执行
- **WHEN** speed_multiplier = 4
- **THEN** 累积时间乘以 4

#### Scenario: 暂停
- **WHEN** is_paused = true
- **THEN** tick 不推进

### Requirement: DriverContext

bevy_adapter SHALL 定义 `DriverContext` 结构，包含 `bevy_cmds: &CommandBuffer` 引用。DriverContext 为只读上下文。

### Requirement: 命令消费契约

每条命令 SHALL 每 tick 只消费一次。`commands_for_tick()` 是过滤读取。已消费命令由 `simulation_driver_system` 统一清理。其他系统不得执行此清理操作。

### Requirement: 录制契约

仅在 Live + 录制开启 + 非 seek 时 SHALL 录制外部命令。AI 命令 SHALL NOT 被录制。async_seek == true 时 SHALL NOT 录制。

### Requirement: Seek 语义

Seek SHALL 在同一 driver 下连续推进多个 tick。向后 seek 重新初始化世界。分帧完成（每帧 500 tick）。Seek 完成后 accumulator = 0.0。Seek 期间 is_seeking = true。

### Requirement: I7 不变量

SimulationDriver SHALL NOT 直接修改 SimulationWorld。唯一合法路径：commands → inject → run_tick。

### Requirement: GameMode 门控

bevy_adapter SHALL 定义 `GameMode` 枚举（Live/Replay），输入系统通过 `not(GameMode::Replay)` 条件在回放时不运行。视觉系统在两种模式都运行。

#### Scenario: 回放时输入系统不运行
- **WHEN** GameMode = Replay
- **THEN** command_issue_system 等输入系统不执行

### Requirement: HashMap 确定性遍历

simulation 层的位置查询 SHALL 使用 HashMap（O(1) 查找），遍历前 SHALL 对 keys 排序保证确定性。enemy_positions SHALL 保留 BTreeMap。

### Requirement: pending.events 清理

simulation_driver_system SHALL 在帧首调用 pending.events.clear()，与旧 tick_driver_system 行为一致。

## ADDED Requirements

### Requirement: SimulationDriver 资源

bevy_adapter SHALL 定义 `SimulationDriver` 资源，包含 `TickClock`（时序）、`SchedulerState`（调度）、`CommandSource`（命令来源）三层。`current_tick` 只在 `TickClock` 中持有，是唯一权威值。

#### Scenario: 初始化为 Live 模式
- **WHEN** 应用启动
- **THEN** SimulationDriver 初始化为 Live 模式，speed_multiplier = 1，is_paused = false

#### Scenario: current_tick 唯一权威
- **WHEN** 任意系统读取当前 tick
- **THEN** 只能从 SimulationDriver.clock.current_tick 读取，无其他 tick 持有者

### Requirement: CommandSource 枚举

bevy_adapter SHALL 定义 `CommandSource` 枚举（Live/Replay），通过 `commands_for_tick(tick, ctx)` 方法获取命令。Live 模式通过 `DriverContext.bevy_cmds` 访问 Bevy CommandBuffer，Replay 模式从 ReplayFile 读取。

#### Scenario: Live 模式命令获取
- **WHEN** SimulationDriver.source 为 Live，tick 为 N
- **THEN** commands_for_tick 返回 Bevy CommandBuffer 中 tick == N 的命令

#### Scenario: Replay 模式命令获取
- **WHEN** SimulationDriver.source 为 Replay，tick 为 N
- **THEN** commands_for_tick 返回 ReplayFile 中 tick N 的命令

### Requirement: simulation_driver_system

bevy_adapter SHALL 实现 `simulation_driver_system`，替代原 `tick_driver_system` 和 `replay_tick_driver_system`。系统 SHALL 按以下顺序执行：获取命令 → 注入 simulation CommandBuffer → 调用 run_tick。每个 tick 严格执行同一流水线，不存在绕过路径。

#### Scenario: 统一流水线
- **WHEN** Live 模式或 Replay 模式
- **THEN** 每个 tick 的执行路径完全相同：commands_for_tick → inject_commands → run_tick

#### Scenario: 系统调度顺序
- **WHEN** GameActive 为 true
- **THEN** simulation_driver_system 在 sync_entities_system 之前执行（.before()）

### Requirement: SchedulerState 调度控制

`SchedulerState` SHALL 包含 is_paused、speed_multiplier、seek_target、async_seek 字段。speed_multiplier 只影响每帧 tick 调度密度，不影响每个 tick 内的命令注入顺序和 run_tick 执行语义。

#### Scenario: 倍速执行
- **WHEN** speed_multiplier = 4
- **THEN** 累积时间乘以 4，每帧处理约 4 个 tick

#### Scenario: 暂停
- **WHEN** is_paused = true
- **THEN** tick 不推进

### Requirement: DriverContext

bevy_adapter SHALL 定义 `DriverContext` 结构，包含 `bevy_cmds: &CommandBuffer` 引用。LiveCommandSource 通过 ctx 访问 Bevy CommandBuffer。DriverContext 为只读上下文。

#### Scenario: Live 命令通过 ctx 获取
- **WHEN** LiveCommandSource 调用 commands_for_tick
- **THEN** 通过 ctx.bevy_cmds 过滤当前 tick 的命令

### Requirement: 命令消费契约

每条命令 SHALL 每 tick 只消费一次。`commands_for_tick()` 是过滤读取，不消费 Bevy CommandBuffer。`run_tick()` 内部的 `consume_commands_system` 执行一次性消费。已消费命令由 `simulation_driver_system` 统一清理。

#### Scenario: 单次消费
- **WHEN** Bevy CommandBuffer 包含 tick N 的命令
- **THEN** 该命令在 tick N 被消费一次，之后从 Bevy CommandBuffer 中移除

### Requirement: Seek 语义

Seek SHALL 在同一 driver 下连续推进多个 tick。向后 seek 重新初始化世界后从 0 推进。向前 seek 从当前位置推进。分帧完成（每帧 500 tick）。Seek 完成后 accumulator = 0.0。Seek 期间 is_seeking = true。

#### Scenario: 向后 seek
- **WHEN** seek_target < current_tick
- **THEN** 重新初始化世界，从 tick 0 快放到目标

#### Scenario: 向前 seek
- **WHEN** seek_target > current_tick
- **THEN** 从当前 tick 快放到目标

#### Scenario: seek 完成后 accumulator 清零
- **WHEN** seek 到达目标 tick
- **THEN** clock.accumulator = 0.0

### Requirement: 录制契约

仅在 Live + 录制开启 + 非 seek 时 SHALL 录制外部命令。AI 命令 SHALL NOT 被录制。async_seek == true 时 SHALL NOT 录制。

#### Scenario: seek 期间不录制
- **WHEN** async_seek = true
- **THEN** ReplayRecorder 不记录任何命令

### Requirement: I7 不变量

SimulationDriver SHALL NOT 直接修改 SimulationWorld。唯一合法路径：commands → inject_commands → run_tick。

#### Scenario: 验证唯一调用点
- **WHEN** 在 bevy_adapter crate 中搜索 run_tick 调用
- **THEN** 只有 simulation_driver_system 中调用 run_tick

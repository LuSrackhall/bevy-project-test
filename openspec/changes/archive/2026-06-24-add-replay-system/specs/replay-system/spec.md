## ADDED Requirements

### Requirement: Replay 文件数据结构

simulation crate SHALL 定义 `ReplayFile` 纯数据结构，包含 `format_version: u32`、`seed: u64`、`map_size: MapSize`、`total_ticks: u32`、`commands_per_tick: BTreeMap<u32, Vec<GameCommand>>`。

#### Scenario: Replay 文件可序列化
- **WHEN** 创建一个包含 seed、map_size 和命令序列的 ReplayFile
- **THEN** 可通过 serde 序列化为 RON 格式并反序列化还原

#### Scenario: Replay 文件可重建仿真
- **WHEN** 使用 ReplayFile 中的 seed 初始化 simulation World
- **AND** 按 tick 顺序注入 commands_per_tick 中的命令并执行 run_tick
- **THEN** 仿真结果与录制时完全一致

### Requirement: SimulationSeed 资源持久化

simulation crate SHALL 定义 `SimulationSeed(pub u64)` 资源，在 `init_simulation_world` 时插入，供录制时读取。

#### Scenario: Seed 可从 World 中读取
- **WHEN** 使用 seed 42 初始化 simulation World
- **THEN** World 中的 `SimulationSeed` 资源值为 42

### Requirement: GameCommand 类型 serde 支持

GameCommand、Action 及其所有依赖类型（Fixed、FixedVec2、UnitId、SoldierType、ShieldState、Faction、SeekScope、SeekDirective）SHALL 派生 `Serialize` 和 `Deserialize`。

#### Scenario: GameCommand 可序列化
- **WHEN** 创建一个包含 MoveTo 动作的 GameCommand
- **THEN** 可通过 serde 序列化为字节并反序列化还原，结果与原值相等

#### Scenario: 所有 Action 变体可序列化
- **WHEN** 创建每个 Action 变体的 GameCommand（MoveTo、Attack、ForceMove、ReturnToCity、SetShield、SetSpawnType、SetSeekStance、NoOp）
- **THEN** 每个变体均可正确序列化和反序列化

### Requirement: ReplayRecorder 录制外部命令

bevy_adapter SHALL 实现 `ReplayRecorder` 资源，在每个 tick 拦截从 Bevy 侧 CommandBuffer 提取的外部玩家命令并记录。AI 命令（在 run_tick 内部产生）SHALL NOT 被录制。

#### Scenario: 录制仅包含外部命令
- **WHEN** 玩家发送 3 条命令，AI 在 run_tick 中产生 2 条命令
- **THEN** ReplayRecorder 仅记录玩家的 3 条命令

#### Scenario: GameOver 时生成 Replay 文件
- **WHEN** 游戏结束（GameActive 从 true 变为 false）
- **AND** 录制功能已开启
- **THEN** ReplayRecorder 将录制数据序列化为 ReplayFile 并写入磁盘

### Requirement: ReplayController 回放控制

bevy_adapter SHALL 实现 `ReplayController` 资源和 `replay_tick_driver_system`，支持暂停、继续、快进（2x/4x）、进度条 seek（从 tick 0 快放到目标 tick）。回放系统与实时 tick 驱动 SHALL 通过 `run_if` 条件互斥运行。

#### Scenario: 暂停和继续
- **WHEN** 回放模式下设置 is_paused = true
- **THEN** tick 停止推进，世界状态冻结
- **WHEN** 设置 is_paused = false
- **THEN** tick 继续推进

#### Scenario: 快进
- **WHEN** 回放模式下设置 speed = Fast4x
- **THEN** 每帧执行 4 个 tick（而非默认 1 个）

#### Scenario: 进度条 seek
- **WHEN** 回放模式下设置 target_tick = 6000（当前在 tick 1000）
- **THEN** 系统从 tick 1 快速重放到 tick 6000，然后恢复播放

### Requirement: GameMode 枚举切换

bevy_adapter SHALL 实现 `GameMode` 枚举（Live/Replay），`tick_driver_system` 在 Live 模式运行，`replay_tick_driver_system` 在 Replay 模式运行，两者互斥。

#### Scenario: 模式互斥
- **WHEN** GameMode 为 Live
- **THEN** 仅 tick_driver_system 运行
- **WHEN** GameMode 为 Replay
- **THEN** 仅 replay_tick_driver_system 运行

### Requirement: ReplayStatus 资源

bevy_adapter SHALL 暴露 `ReplayStatus { is_replay: bool, total_ticks: u32 }` 资源供 render_view 读取进度条数据。render_view SHALL NOT import 任何 simulation 类型。

#### Scenario: 进度条数据可读
- **WHEN** 处于回放模式，当前 tick 为 3000，总 tick 为 12000
- **THEN** render_view 可通过 ReplayStatus 和 TickClock 读取到 is_replay=true, current_tick=3000, total_ticks=12000

### Requirement: Replay 播放器 UI

render_view SHALL 实现 Replay 播放器 UI，包含快退/快进 10 秒按钮、播放/暂停按钮、速度控制（1x/2x/4x/8x/16x）、进度条（纯视觉显示，不支持拖拽 seek）、M:SS 时间显示。

#### Scenario: 主菜单加载 Replay
- **WHEN** 用户在主菜单点击 "Load Replay" 按钮
- **AND** 选择一个 .replay 文件
- **THEN** 进入 Replay 模式，从 tick 0 开始播放

#### Scenario: 录制开关设置
- **WHEN** 用户在设置中关闭 "自动录制 Replay"
- **THEN** GameOver 时 ReplayRecorder 不写入文件

### Requirement: Replay 文件格式版本化

ReplayFile 的 `format_version` 字段 SHALL 从 1 开始，每次格式变更递增。加载时如果版本不匹配 SHALL 显示警告。

#### Scenario: 版本兼容检查
- **WHEN** 加载 format_version = 99 的 Replay 文件（当前支持 version 1）
- **THEN** 显示版本不匹配警告，拒绝加载

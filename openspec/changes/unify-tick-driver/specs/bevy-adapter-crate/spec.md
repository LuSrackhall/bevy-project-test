## MODIFIED Requirements

### Requirement: Tick 调度驱动

`tick_driver_system` 和 `replay_tick_driver_system` SHALL 被删除，替换为统一的 `simulation_driver_system`。`ReplayController` 资源 SHALL 被删除，功能合并到 `SimulationDriver`。

#### Scenario: 统一驱动
- **WHEN** 应用运行
- **THEN** 只有 simulation_driver_system 驱动 tick 推进

### Requirement: Tick 命令快照

录制逻辑 SHALL 移入 `simulation_driver_system`。在获取命令后、注入 simulation 前，如 Live + 录制开启 + 非 seek，SHALL 将命令副本保存到 ReplayRecorder。

### Requirement: TickClock 兼容

TickClock SHALL 作为独立 Resource 注册，由 simulation_driver_system 每帧同步。

## ADDED Requirements

### Requirement: GameActive + GameMode 双重门控

simulation_driver_system 和 sync_entities_system SHALL 通过 `GameActive(true)` 门控。输入系统 SHALL 通过 `GameActive(true) && !GameMode::Replay` 门控。

#### Scenario: 回放时输入系统不运行
- **WHEN** GameMode = Replay
- **THEN** command_issue_system 等不执行，防止干扰仿真

### Requirement: GameMode 资源

bevy_adapter SHALL 定义 `GameMode` 枚举（Live/Replay），作为轻量门控。reset_game_system 启动新游戏时 SHALL 设为 Live。加载 Replay 时 SHALL 设为 Replay。cleanup_playing_system SHALL 重置为 Live。

#### Scenario: 新游戏 Live 模式
- **WHEN** 用户从主菜单开始新游戏
- **THEN** GameMode = Live，输入系统正常运行

#### Scenario: 加载 Replay
- **WHEN** 用户从主菜单加载 Replay 文件
- **THEN** GameMode = Replay，输入系统不运行

### Requirement: SimulationDriver 初始化

SimulationDriver SHALL 通过 `insert_resource(SimulationDriver::new_live())` 注册。

### Requirement: world_fingerprint 工具

bevy_adapter SHALL 保留 `world_fingerprint` 函数（`#[allow(dead_code)]`），用于确定性调试。

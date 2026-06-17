# game-lifecycle

## Purpose

游戏生命周期管理：GameState 状态机、NeedsGameReset 标志、reset_game_system 重置逻辑、SimulationWorld 延迟初始化。

## ADDED Requirements

### Requirement: GameState 三变体枚举
`GameState` SHALL 定义为 3 个变体的 Bevy `States` 枚举：`MainMenu`（默认）、`Playing`、`GameOver`。`Paused` SHALL NOT 作为 `GameState` 的变体存在。

#### Scenario: 初始状态
- **WHEN** App 启动
- **THEN** `GameState` 初始值为 `MainMenu`

#### Scenario: 无 Paused 变体
- **WHEN** 审查 `GameState` 枚举定义
- **THEN** 枚举仅包含 `MainMenu`、`Playing`、`GameOver` 三个变体

### Requirement: NeedsGameReset 标志
`NeedsGameReset(bool)` SHALL 作为 Bevy `Resource` 存在，默认值为 `false`。按钮处理器在触发游戏开始/重开时 SHALL 将其设为 `true`，`reset_game_system` 在完成重置后 SHALL 将其设为 `false`。

#### Scenario: 首次开始游戏
- **WHEN** 用户在 MainMenu 点击"单人模式"，按钮处理器设 `NeedsGameReset(true)` 并切换 `GameState` 到 `Playing`
- **THEN** `OnEnter(Playing)` 触发 `reset_game_system`，检测到 `NeedsGameReset.0 == true`，执行完整仿真重置

#### Scenario: 暂停恢复不触发重置
- **WHEN** 用户在暂停菜单点击"继续"，`Paused` 设为 `false`，`GameState` 不变
- **THEN** `NeedsGameReset` 保持 `false`，不触发任何重置

#### Scenario: 重开游戏触发重置
- **WHEN** 用户在 GameOver 点击"再来一局"，设 `NeedsGameReset(true)` 并切到 `MainMenu`，然后点"单人模式"进入 `Playing`
- **THEN** `reset_game_system` 执行完整重置：销毁旧实体、清空状态、用新种子创建 SimulationWorld

### Requirement: reset_game_system 生命周期
`reset_game_system` SHALL 在 `OnEnter(GameState::Playing)` 时执行。若 `NeedsGameReset == true`，SHALL 执行以下操作序列：
1. 销毁所有 `LogicEntityRef` 实体
2. 清空 `UnitIdMapper`
3. 重置 `TickClock` 为默认值
4. 清空 `CommandBuffer`
5. 清空 `PendingEvents`
6. 重置 `SelectionState`
7. 用 `SystemTime` 种子创建新 `SimulationWorld` 并生成地图
8. 设 `NeedsGameReset.0 = false`
9. 设 `Paused.0 = false`

若 `NeedsGameReset == false`，SHALL 仅设 `Paused.0 = false`，不执行重置。

#### Scenario: 完整重置序列
- **WHEN** `OnEnter(Playing)` 触发，`NeedsGameReset.0 == true`
- **THEN** 所有旧 `LogicEntityRef` 实体被销毁，`UnitIdMapper` 为空，`TickClock.current_tick == 0`，`SimulationWorld` 包含新种子的新地图

#### Scenario: 重置后 HUD 设置
- **WHEN** `reset_game_system` 完成后
- **THEN** `setup_hud` SHALL 在 `reset_game_system` 之后执行（通过 `.after()` 保证顺序），创建新的 HUD UI

### Requirement: cleanup_playing 系统
`cleanup_playing` SHALL 在 `OnExit(GameState::Playing)` 时执行，销毁所有 HUD 实体和暂停菜单 UI。

#### Scenario: 退出到 MainMenu
- **WHEN** `GameState` 从 `Playing` 切换到 `MainMenu`
- **THEN** `OnExit(Playing)` 触发，所有 HUD 实体和暂停菜单 UI 被销毁

#### Scenario: 退出到 GameOver
- **WHEN** `GameState` 从 `Playing` 切换到 `GameOver`
- **THEN** `OnExit(Playing)` 触发，所有 HUD 实体和暂停菜单 UI 被销毁

### Requirement: SimulationWorld 延迟初始化
`SimulationWorld` SHALL NOT 在 `main.rs` 启动时通过 `init_sim_world()` 初始化。App 启动时 SHALL 插入一个空的 `SimulationWorld`（由 `init_simulation_world` 创建但未调用 `generate_map`）。仿真世界仅在 `reset_game_system` 中首次进入 `Playing` 时初始化。

#### Scenario: App 启动时无仿真
- **WHEN** App 启动，`GameState == MainMenu`
- **THEN** `SimulationWorld` 资源存在但为空（无地图实体），`tick_driver_system` 不运行（`run_if` 守卫）

#### Scenario: 首次进入游戏
- **WHEN** 用户点击"单人模式"，`GameState` 切换到 `Playing`
- **THEN** `reset_game_system` 用随机种子创建完整 SimulationWorld，地图实体被生成

## MODIFIED Requirements

### Requirement: 主菜单系统
render_view crate SHALL 提供主菜单界面（游戏标题、"单人模式"按钮），在 `GameState::MainMenu` 时显示。点击"单人模式" SHALL 设 `NeedsGameReset(true)` 并切换 `GameState` 到 `Playing`。

#### Scenario: 进入游戏
- **WHEN** 玩家在主菜单点击"单人模式"
- **THEN** `NeedsGameReset` 设为 `true`，`GameState` 切换到 `Playing`，`OnExit(MainMenu)` 清除主菜单 UI，`OnEnter(Playing)` 触发 `reset_game_system` 执行完整重置

### Requirement: 暂停菜单系统
暂停菜单 SHALL 基于 `Paused(bool)` 资源控制可见性，SHALL NOT 依赖 `GameState` 状态转换。暂停菜单在 `setup_hud` 中创建（初始 Hidden），通过 `update_pause_visibility` 系统切换可见性。

#### Scenario: 暂停后继续
- **WHEN** 玩家在暂停菜单点击"继续"
- **THEN** `Paused` 设为 `false`，暂停菜单隐藏，游戏恢复

#### Scenario: Esc 行为
- **WHEN** 当前有选中的士兵，玩家按 Esc
- **THEN** 选区被清除（不进入暂停）。再次按 Esc 才设 `Paused(true)`

### Requirement: 结算画面系统
render_view crate SHALL 提供结算画面，在 `GameState::GameOver` 时显示。"再来一局"按钮 SHALL 设 `NeedsGameReset(true)` 并切到 `MainMenu`。

#### Scenario: 玩家胜利
- **WHEN** 所有敌方城池被消灭，`check_victory_system` 检测到敌方无实体
- **THEN** `GameState` 切换到 `GameOver`，`OnExit(Playing)` 销毁 HUD，结算画面显示

#### Scenario: 再来一局
- **WHEN** 玩家点击"再来一局"
- **THEN** `NeedsGameReset` 设为 `true`，`GameState` 切换到 `MainMenu`，用户点击"单人模式"后进入新游戏

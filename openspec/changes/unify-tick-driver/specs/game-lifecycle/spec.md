## MODIFIED Requirements

### Requirement: GameState 枚举

GameState SHALL 保持不变（MainMenu/Playing/GameOver）。Replay 模式通过 SimulationDriver + GameMode 控制，不通过 GameState。

#### Scenario: Replay 进入 Playing
- **WHEN** 用户加载 Replay 文件
- **THEN** GameState 切换为 Playing，GameMode 设为 Replay，SimulationDriver 设为 Replay 模式

### Requirement: 输入系统门控

游戏输入系统（selection_click、command_issue、seek_stance_shortcut 等）SHALL 在 GameMode::Replay 时不运行。视觉系统（debug_shape、unit_info_bar 等）SHALL 在两种模式都运行。

#### Scenario: 回放时视觉正常但不可操作
- **WHEN** GameMode = Replay 且 GameState = Playing
- **THEN** 城市、士兵、战斗动画正常显示，但鼠标点击不会产生命令

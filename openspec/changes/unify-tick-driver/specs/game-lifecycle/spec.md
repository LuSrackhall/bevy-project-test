## MODIFIED Requirements

### Requirement: GameState 枚举

GameState SHALL 保持不变（MainMenu/Playing/GameOver）。Replay 模式通过 SimulationDriver.source 控制，不通过 GameState。

#### Scenario: Replay 进入 Playing
- **WHEN** 用户加载 Replay 文件
- **THEN** GameState 切换为 Playing，SimulationDriver.source 设为 Replay

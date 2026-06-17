## MODIFIED Requirements

### Requirement: GameOver panel action buttons
结算面板 SHALL 包含两个操作按钮：
- "再来一局"：设 `NeedsGameReset(true)` 并切换 `GameState` 到 `MainMenu`（用户再点"单人模式"开始新游戏）
- "返回主菜单"：切换 `GameState` 到 `MainMenu`

#### Scenario: Restart game
- **WHEN** 玩家在结算面板点击 "再来一局"
- **THEN** `NeedsGameReset` 设为 `true`，`GameState` 切换为 `MainMenu`，`OnExit(GameOver)` 清理结算面板，主菜单显示。用户点击"单人模式"后 `OnEnter(Playing)` 触发完整重置

#### Scenario: Return to main menu
- **WHEN** 玩家在结算面板点击 "返回主菜单"
- **THEN** `GameState` 切换为 `MainMenu`，`OnExit(GameOver)` 清理结算面板，主菜单重新显示

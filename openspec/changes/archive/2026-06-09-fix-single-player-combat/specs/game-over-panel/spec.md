## ADDED Requirements

### Requirement: GameOver panel on victory or defeat
GameState 变为 GameOver 时，SHALL 显示结算面板，展示胜负结果和统计数据。

#### Scenario: Player wins
- **WHEN** check_victory_system 检测敌方城池数为 0，设置 GameState::GameOver
- **THEN** 结算面板显示 "胜利！"、游戏时间、剩余城池数、总击杀数

#### Scenario: Player loses
- **WHEN** check_victory_system 检测己方城池数为 0，设置 GameState::GameOver
- **THEN** 结算面板显示 "失败！"、游戏时间、剩余城池数、总击杀数

### Requirement: GameOver panel action buttons
结算面板 SHALL 包含两个操作按钮：
- "再来一局"：切换到 Playing 状态（触发 OnEnter 重新生成地图和清理旧实体）
- "返回主菜单"：切换到 MainMenu 状态

#### Scenario: Restart game
- **WHEN** 玩家在结算面板点击 "再来一局"
- **THEN** GameState 切换为 Playing，cleanup_game 清理旧实体，OnEnter(Playing) 触发地图生成

#### Scenario: Return to main menu
- **WHEN** 玩家在结算面板点击 "返回主菜单"
- **THEN** GameState 切换为 MainMenu，OnExit(GameOver) 清理结算面板，主菜单重新显示

### Requirement: Game statistics tracked
游戏 SHALL 跟踪游戏开始时间和累计击杀数。

#### Scenario: Kill tracking
- **WHEN** 玩家士兵击杀一名敌方士兵（SoldierDiedEvent 触发，killer faction == Player）
- **THEN** GameStats.total_kills += 1

#### Scenario: Elapsed time calculation
- **WHEN** GameOver 面板渲染
- **THEN** 游戏时间 = 当前 elapsed_seconds - GameStats.start_time

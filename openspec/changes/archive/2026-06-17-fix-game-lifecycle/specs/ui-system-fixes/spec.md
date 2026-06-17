## MODIFIED Requirements

### Requirement: Pause button in top bar works
顶部状态栏的暂停按钮 SHALL 点击时设 `Paused(true)`，SHALL NOT 切换 `GameState`。

#### Scenario: Click pause button
- **WHEN** 玩家点击顶部状态栏的暂停按钮
- **THEN** `Paused` 设为 `true`，暂停菜单 UI 可见性切换为 `Visible`，`GameState` 不变

### Requirement: Top status bar updates in real time
顶部状态栏 SHALL 每帧显示最新数据：玩家城池数/总城池数、玩家总人口、已运行时间（mm:ss 格式）。HUD 更新系统 SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时运行。

#### Scenario: Status bar shows player stats
- **WHEN** 游戏 Playing 状态中且未暂停，玩家拥有 3 座城池、总计 25 人口
- **THEN** 顶部状态栏显示 "城 3/N"（N 为总城池数）、"兵 25"、"T MM:SS"（已运行时间）

#### Scenario: 暂停时不更新
- **WHEN** `GameState::Playing` 且 `Paused.0 == true`
- **THEN** HUD 更新系统不运行

## ADDED Requirements

### Requirement: Paused 布尔资源
`Paused(bool)` SHALL 作为 Bevy `Resource` 存在，默认值为 `false`。暂停/继续操作 SHALL 仅翻转此布尔值，SHALL NOT 触发任何 `OnEnter`/`OnExit` 状态转换。

#### Scenario: 暂停游戏
- **WHEN** 用户在 `GameState::Playing` 且无选区时按 Escape
- **THEN** `Paused.0` 设为 `true`，`GameState` 不变，暂停菜单 UI 可见性切换为 `Visible`

#### Scenario: 继续游戏
- **WHEN** 用户在暂停状态点击"继续"
- **THEN** `Paused.0` 设为 `false`，`GameState` 不变，暂停菜单 UI 可见性切换为 `Hidden`

### Requirement: 暂停时仿真不 tick
`tick_driver_system` 和 `sync_entities_system` SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时运行。

#### Scenario: 暂停时 tick 停止
- **WHEN** `GameState::Playing` 且 `Paused.0 == true`
- **THEN** `tick_driver_system` 不执行，`TickClock.current_tick` 不递增

#### Scenario: 继续后 tick 恢复
- **WHEN** `Paused` 从 `true` 变为 `false`，`GameState` 仍为 `Playing`
- **THEN** `tick_driver_system` 恢复执行，从暂停前的 `TickClock` 状态继续

### Requirement: 暂停时胜利检查禁用
`check_victory_system` SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时运行。

#### Scenario: 暂停期间不触发 GameOver
- **WHEN** 游戏暂停，且仿真层中玩家最后的单位在理论上下一 tick 会被消灭
- **THEN** `check_victory_system` 不运行，`GameState` 不变为 `GameOver`

#### Scenario: 继续后检测胜负
- **WHEN** 暂停解除，`check_victory_system` 恢复运行
- **THEN** 若一方已被全灭，`GameState` 切换到 `GameOver`

### Requirement: 暂停 UI 由资源驱动
暂停菜单 UI SHALL 在 `setup_hud` 中创建（初始 `Visibility::Hidden`），通过 `update_pause_visibility` 系统根据 `Paused` 资源切换可见性。

#### Scenario: 暂停菜单创建
- **WHEN** `OnEnter(Playing)` 触发 `setup_hud`
- **THEN** 暂停菜单 UI 实体被创建，初始 `Visibility::Hidden`

#### Scenario: 暂停时显示
- **WHEN** `Paused.0` 变为 `true`
- **THEN** `update_pause_visibility` 将暂停菜单可见性设为 `Visible`

#### Scenario: 继续时隐藏
- **WHEN** `Paused.0` 变为 `false`
- **THEN** `update_pause_visibility` 将暂停菜单可见性设为 `Hidden`

### Requirement: 暂停菜单按钮行为
暂停菜单 SHALL 包含三个按钮，行为如下：
- "继续"：设 `Paused(false)`
- "重新开始"：设 `Paused(false)`、`NeedsGameReset(true)`、`NextState(MainMenu)`
- "主菜单"：设 `Paused(false)`、`NextState(MainMenu)`

#### Scenario: 重新开始
- **WHEN** 用户点击"重新开始"
- **THEN** `Paused` 设为 `false`，`NeedsGameReset` 设为 `true`，`GameState` 切换到 `MainMenu`，`OnExit(Playing)` 销毁 HUD 和暂停菜单

#### Scenario: 返回主菜单
- **WHEN** 用户点击"主菜单"
- **THEN** `Paused` 设为 `false`，`GameState` 切换到 `MainMenu`

### Requirement: 暂停输入处理
`handle_pause_input` 系统 SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时响应 Escape 键。行为优先级：
1. 若 `SeekPanelState.input_active == true`，关闭输入面板
2. 若有选中的单位或城池，清除选区
3. 否则设 `Paused(true)`

#### Scenario: 有选区时按 Esc
- **WHEN** 选中了 3 个士兵，按 Escape
- **THEN** 选区被清除，不进入暂停

#### Scenario: 无选区时按 Esc
- **WHEN** 无选中单位，按 Escape
- **THEN** `Paused` 设为 `true`，暂停菜单显示

#### Scenario: 暂停时按 Esc
- **WHEN** 游戏已暂停，按 Escape
- **THEN** 无效果（`handle_pause_input` 不在暂停时运行）

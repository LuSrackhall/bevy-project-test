## ADDED Requirements

### Requirement: Lobby 等待室 UI

Lobby 状态具有完整的 UI 渲染，显示当前连接阶段和状态。

- 使用 Bevy 0.19 UI（Node + Text + WidgetButton + On<Activate> observer）
- 显示当前阶段：Connecting / Connected / Failed
- 取消按钮，点击后返回 `GameState::MainMenu`
- 使用 `on_exit` 系统清理所有 Lobby UI 实体

#### Scenario: Lobby 显示连接中状态

- **WHEN** 进入 `GameState::Lobby`，`LobbyConnectionStatus.result` 为 `None`
- **THEN** UI 显示 "正在连接..." 以及取消按钮

#### Scenario: Lobby 显示连接成功状态

- **WHEN** `LobbyConnectionStatus.result` 为 `Some(Ok(()))`
- **THEN** UI 显示 "已连接，等待其他玩家..." 以及取消按钮

#### Scenario: 取消按钮返回主菜单

- **WHEN** 玩家点击取消按钮
- **THEN** 状态切换为 `GameState::MainMenu`，网络资源被清理

#### Scenario: 连接失败显示错误

- **WHEN** `LobbyConnectionStatus.result` 为 `Some(Err(msg))`
- **THEN** UI 显示错误信息以及 "返回主菜单" 按钮

### Requirement: Lobby UI 在退出时清理

- 使用 `OnExit(GameState::Lobby)` 系统，移除所有带 `LobbyUI` 标记的实体

#### Scenario: Lobby UI 清理

- **WHEN** `OnExit(GameState::Lobby)` 触发
- **THEN** 所有 `LobbyUI` 实体被销毁

## ADDED Requirements

### Requirement: TCP 连接异步轮询

TCP 连接过程不阻塞 Bevy 主线程。发起连接后，连接状态通过 `LobbyConnectionStatus` Resource 异步通知。

- `LobbyConnectionStatus` 包含 `Arc<Mutex<Option<Result<(), String>>>>`
- tokio 线程在 TCP 连接成功后设置 `Some(Ok(()))`，失败后设置 `Some(Err(msg))`
- 每帧轮询一次，不忙等

#### Scenario: 连接成功

- **WHEN** tokio 线程的 TCP 连接成功
- **THEN** `LobbyConnectionStatus.result` 变为 `Some(Ok(()))`

#### Scenario: 连接失败

- **WHEN** tokio 线程的 TCP 连接失败或超时
- **THEN** `LobbyConnectionStatus.result` 变为 `Some(Err(错误信息))`

### Requirement: 主菜单联机按钮指向 Lobby

主菜单的"开始联机"按钮将状态设置为 `GameState::Lobby`，而非 `GameState::Playing`。

#### Scenario: 联机按钮

- **WHEN** 玩家点击"联机"按钮
- **THEN** `NeedsGameReset` 设为 `Network`，`GameState` 设为 `Lobby`

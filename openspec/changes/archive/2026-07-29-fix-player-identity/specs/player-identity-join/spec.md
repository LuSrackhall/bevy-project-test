## ADDED Requirements

### Requirement: GameJoined 更新 NetworkGameStart

当 `lobby_update_system` 收到 `NetworkEvent::GameJoined`，
必须同步更新 `NetworkGameStart.player_id` 和 `NetworkGameStart.player_count`。

#### Scenario: GameJoined 到达
- **WHEN** 加入者收到 `RelayServerMessage::GameJoined { player_id: 1, player_count: 2 }`
- **THEN** `NetworkGameStart.player_id` 必须设为 `1`，`NetworkGameStart.player_count` 必须设为 `2`

#### Scenario: reset_game_system 使用正确值
- **WHEN** `reset_game_system` 在 GameJoined 之后运行
- **THEN** 创建的 `LocalPlayerId` 必须等于 relay 分配的 player_id

### Requirement: max_players 来自房间信息

`handle_join_room` 必须使用 `JoinRoomRequest.max_players` 而非硬编码。

#### Scenario: 观察者设置 max_players
- **WHEN** 加入按钮被点击
- **THEN** `JoinRoomRequest.max_players` 必须从发现包的 `room.max_players` 读取

#### Scenario: 创建 PlayerSlots
- **WHEN** `reset_game_system` 创建 `PlayerSlots`
- **THEN** `PlayerSlots::multi_player` 的 player_count 必须等于 relay 分配的 player_count

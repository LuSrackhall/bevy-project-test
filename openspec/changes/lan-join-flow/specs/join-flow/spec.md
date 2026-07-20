## ADDED Requirements

### Requirement: JoinRoomRequest Resource

UI 设置 `JoinRoomRequest` 触发加入流程，Integration System 消费。

#### Scenario: Request triggers join

- **WHEN** `JoinRoomRequest.requested` 设为 `true`，包含有效的 `endpoint`/`room_id`/`relay_id`
- **THEN** Integration System 读取请求，启动 TCP 连接，发送 `JoinGame`

### Requirement: JoinGame protocol

Relay 验证加入请求并分配 player_id。

#### Scenario: Join succeeds

- **WHEN** Relay 收到 `JoinGame { room_id, relay_id }`，身份验证通过，有空闲 slot
- **THEN** 分配 `player_id`，返回 `GameJoined { player_id, player_count }`
- **AND** `player_id` 在 Session 生命周期内唯一

#### Scenario: Room full

- **WHEN** Relay 中 `current_players >= max_players`
- **THEN** 返回 `JoinRejected { reason }`，不分配 slot

#### Scenario: Relay identity mismatch

- **WHEN** `JoinGame.relay_id` 与 Relay 自身的 `relay_id` 不匹配
- **THEN** 返回 `JoinRejected { reason }`

### Requirement: LocalPlayerIdentity Resource

Client 在收到 `GameJoined` 后写入 `LocalPlayerIdentity`，作为玩家身份的权威来源。

#### Scenario: Written after GameJoined

- **WHEN** Client 收到 `GameJoined { player_id, player_count }`
- **THEN** 创建 `LocalPlayerIdentity { player_id, player_count }`

### Requirement: NetworkCommandSource creation

NetworkCommandSource 在收到 GameJoined 后创建，使用 Relay 分配的 player_id。

#### Scenario: Created after identity established

- **WHEN** `LocalPlayerIdentity` 已写入
- **THEN** `NetworkCommandSource` 使用 `LocalPlayerIdentity.player_id` 初始化

### Requirement: NeedsGameReset.Network simplified

NeedsGameReset::Network 不再包含 player_id 字段。

#### Scenario: player_id removed

- **WHEN** 创建 `NeedsGameReset::Network`
- **THEN** 只包含 `relay_addr` 和 `player_count`，不包含 `player_id`

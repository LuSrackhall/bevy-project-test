## ADDED Requirements

### Requirement: GameJoined event handling

transport.rs 中的 tokio 线程收到 `GameJoined` 后通过 `NetworkEventReceiver` 推送到 Bevy 主线程。

#### Scenario: Event pushed after GameJoined

- **WHEN** tokio 线程收到 `RelayServerMessage::GameJoined { player_id, player_count }`
- **THEN** 通过 `event_receiver.push(NetworkEvent::GameJoined { player_id, player_count })` 推送

#### Scenario: lobby_update_system consumes event

- **WHEN** 主线程 `lobby_update_system` 读取到 `NetworkEvent::GameJoined`
- **THEN** 更新 `SimulationDriver.source`（需转换为 `NetworkCommandSource`）的 `player_id`
- **AND** 更新 `LocalPlayerIdentity` Resource

## MODIFIED Requirements

### Requirement: Lobby waiting room UI

The lobby waiting room SHALL display a live player list showing all connected players.

#### Scenario: Player list updates on LobbyUpdate
- **WHEN** `NetworkEvent::LobbyUpdate` is received
- **THEN** the player list SHALL be re-rendered to match the updated `LobbyPlayerList`

#### Scenario: Non-host sees "Ready" button
- **WHEN** a non-host player (`IsHost = false`) is in the lobby
- **THEN** the player SHALL see a "就绪" (Ready) button
- **AND** clicking it SHALL send `LobbyReady(true)`
- **AND** the button SHALL update to show "已就绪" (Ready) state

#### Scenario: Host sees "Start Game" button
- **WHEN** a host (`IsHost = true`) is in the lobby
- **THEN** the host SHALL see a "开始游戏" (Start Game) button
- **AND** clicking it SHALL send `LobbyReady(true)` for the host

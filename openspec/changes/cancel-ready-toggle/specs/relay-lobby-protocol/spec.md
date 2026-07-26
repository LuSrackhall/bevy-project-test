## MODIFIED Requirements

### Requirement: Cancel ready status

Players SHALL be able to toggle their ready status in the lobby. The relay SHALL correctly handle `ready: false` in `LobbyReady` messages.

#### Scenario: Cancel ready on server side
- **WHEN** a `LobbyReady { ready: false }` message is received by the relay
- **AND** the game has not started yet
- **THEN** the relay SHALL clear the player's ready bit
- **AND** broadcast an updated `LobbyUpdate` to all clients

#### Scenario: Cancel ready after game start
- **WHEN** a `LobbyReady { ready: false }` message is received by the relay
- **AND** the game has already started
- **THEN** the relay SHALL ignore the message

#### Scenario: Toggle ready button
- **WHEN** a non-host player clicks the ready button while in "就绪" state
- **THEN** the client SHALL send `LobbyReady { ready: false }`
- **AND** update the button text back to "就绪"

#### Scenario: Ready button text updates
- **WHEN** `ReadyState` is `true`
- **THEN** the button SHALL display "已就绪"
- **WHEN** `ReadyState` is `false`
- **THEN** the button SHALL display "就绪"

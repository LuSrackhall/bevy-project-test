# relay-lobby-protocol Specification

## Purpose
TBD - created by archiving change relay-lobby-protocol. Update Purpose after archive.
## Requirements
### Requirement: LobbyReady message

The client SHALL send `RelayClientMessage::LobbyReady` to signal readiness. The relay SHALL track ready states per player.

#### Scenario: All players ready starts game
- **WHEN** all connected players have sent `LobbyReady{ready: true}`
- **THEN** the relay SHALL broadcast `RelayServerMessage::GameStarted` to all players

### Requirement: LobbyUpdate broadcast

After any LobbyReady update, the relay SHALL broadcast `RelayServerMessage::LobbyUpdate` with current player states.

#### Scenario: Lobby state broadcast after ready
- **WHEN** a player sends `LobbyReady`
- **THEN** the relay SHALL send `LobbyUpdate` with all players' ready states to all connected clients


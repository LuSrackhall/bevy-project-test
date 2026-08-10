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

The client SHALL NOT discard a `GameStarted` that arrives in the same control-channel batch as a ready `LobbyUpdate`. The relay broadcasts `LobbyUpdate` then `GameStarted` back-to-back when the last player readies; the client lobby must process both, completing the Lobby → Playing transition.

#### Scenario: GameStarted in same batch as ready LobbyUpdate

- **WHEN** a client drains a batch containing `LobbyUpdate` (local player ready) followed by `GameStarted`
- **THEN** the client SHALL complete the transition to `Playing` (SHALL NOT drop the `GameStarted`)

#### Scenario: only LobbyUpdate in batch

- **WHEN** a batch contains only a `LobbyUpdate` with the local player ready (no `GameStarted`)
- **THEN** the client SHALL transition to the Ready lobby phase, and SHALL consume a `GameStarted` from a later batch to start the game


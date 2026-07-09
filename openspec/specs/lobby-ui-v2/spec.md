# lobby-ui-v2 Specification

## Purpose
TBD - created by archiving change lobby-hall-ui-v2. Update Purpose after archive.
## Requirements
### Requirement: LobbyUpdate event channel

The system SHALL deliver `LobbyUpdate` messages from the tokio thread to the Bevy main thread via the existing `NetworkEventReceiver`.

- `NetworkEvent` SHALL gain a `LobbyUpdate` variant
- `run_session()` SHALL push LobbyUpdate events to `event_receiver`

#### Scenario: LobbyUpdate drives UI state
- **WHEN** a `LobbyUpdate` message arrives from the relay
- **THEN** the Bevy lobby update system SHALL poll `NetworkEventReceiver`
- **AND** update the UI to reflect current player ready states

### Requirement: LobbyReady send mechanism

The Bevy lobby UI SHALL be able to send a `LobbyReady` signal to the relay via `NetworkSender.send_lobby_ready()`.

#### Scenario: Ready button triggers relay message
- **WHEN** the local player clicks the Ready button in the lobby
- **THEN** `NetworkSender.send_lobby_ready()` SHALL enqueue a LobbyReady message
- **AND** the write task SHALL send it to the relay


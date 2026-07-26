## MODIFIED Requirements

### Requirement: JoinGame protocol

The client SHALL send `RelayClientMessage::JoinGame` immediately after TCP connection to the relay, before sending any other messages.

#### Scenario: Client sends JoinGame on connect
- **WHEN** a TCP connection to the relay is established
- **THEN** the client SHALL send `RelayClientMessage::JoinGame { room_id, relay_id }`
- **AND** the relay SHALL validate the relay_id and assign a player_id

### Requirement: Host enters lobby

The room host (the client that created the room) SHALL automatically join the lobby after successful room creation.

#### Scenario: Host joins own lobby
- **WHEN** `handle_create_room` succeeds
- **THEN** the host SHALL set `NeedsGameReset::Network` with the relay endpoint
- **AND** transition to `GameState::Lobby`
- **AND** connect to the local relay via TCP as player 0

### Requirement: LobbyUpdate handling

When `NetworkEvent::LobbyUpdate` is received, the system SHALL extract the player list and check only the local player's ready status.

#### Scenario: LobbyUpdate updates player list
- **WHEN** `NetworkEvent::LobbyUpdate { players }` is received
- **THEN** the `players` SHALL be stored in a `LobbyPlayerList` resource
- **AND** the system SHALL look up the local player's `ready` status
- **AND** set `LobbyPhase::Ready` only if the local player is ready

### Requirement: Host identity

The system SHALL track whether the local client is the room host via a dedicated `IsHost` resource, instead of inferring from player_id.

#### Scenario: Host flag set on room creation
- **WHEN** a room is created via `handle_create_room`
- **THEN** `IsHost(true)` SHALL be inserted as a resource
- **WHEN** a room is joined via `handle_join_room`
- **THEN** `IsHost(false)` SHALL be inserted as a resource

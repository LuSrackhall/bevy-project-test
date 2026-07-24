## ADDED Requirements

### Requirement: Shared relay runtime

The system SHALL provide a shared relay runtime module (`bevy_adapter::relay_core`) that can be used by both `ThreadRelayRuntime` (embedded in the game process) and the standalone `relay` CLI binary. The runtime SHALL handle TCP connection acceptance, client lifecycle, and tick broadcast.

The runtime SHALL support graceful shutdown via an `AtomicBool` stop signal.

#### Scenario: Relay accepts TCP connections
- **WHEN** `run_relay()` is called with a bound `TcpListener`
- **THEN** it SHALL accept incoming TCP connections in a loop
- **AND** spawn a handler task for each accepted connection

#### Scenario: Relay stops on signal
- **WHEN** the `AtomicBool` stop flag is set to `true`
- **THEN** `run_relay()` SHALL exit its accept loop
- **AND** return control to the caller

#### Scenario: Client joins via JoinGame
- **WHEN** a client sends `RelayClientMessage::JoinGame`
- **THEN** the relay SHALL call `RelayServer::on_join_game()`
- **AND** if accepted, send `RelayServerMessage::GameJoined` with assigned `player_id`
- **AND** if rejected, send `RelayServerMessage::JoinRejected`

#### Scenario: Tick collection and broadcast
- **WHEN** `RelayClientMessage::PlayerTick` is received from any connected client
- **THEN** the relay SHALL pass it to `RelayServer::on_player_frame()`
- **AND** if a tick is finalized, broadcast `RelayServerMessage::Broadcast` to all clients

#### Scenario: GameStarted on all players connected
- **WHEN** `RelayServer::on_player_frame()` returns `game_just_started = true`
- **THEN** the relay SHALL broadcast `RelayServerMessage::GameStarted` to all clients

#### Scenario: Client disconnect cleanup
- **WHEN** a TCP connection is closed by a client
- **THEN** the relay SHALL call `RelayServer::on_disconnect()`
- **AND** abort the client's writer task

## ADDED Requirements

### Requirement: SingleInitializer

The single::initialize() function SHALL return immediately with no I/O. Live mode requires no initialization parameters.

#### Scenario: single init

- **WHEN** single::initialize() is called
- **THEN** it SHALL return `Ok(() )`

### Requirement: ReplayInitializer

The replay::initialize() function SHALL load a ReplayFile from the path provided in SessionConfig.

#### Scenario: replay init succeeds

- **WHEN** the path exists and contains a valid ReplayFile
- **THEN** it SHALL return `Ok(replay_file)`

#### Scenario: replay init fails

- **WHEN** the path does not exist or the file is invalid
- **THEN** it SHALL return `Err(String)`

### Requirement: NetworkInitializer

The network::initialize() function SHALL:
1. Establish TCP connection to the relay
2. Complete JoinGame/GameJoined handshake
3. Return `NetworkBootstrapResult { player_id, receiver, sender, handle }`

#### Scenario: handshake succeeds

- **WHEN** relay is reachable and responds within timeout
- **THEN** it SHALL return `Ok(NetworkBootstrapResult)` with an assigned player_id

#### Scenario: handshake timeout or failure

- **WHEN** relay is unreachable or times out (5s default)
- **THEN** it SHALL return `Err(String)` AND ensure all resources are cleaned up (no ghost threads)

### Requirement: Initializer does NOT construct CommandSource

initializer returns only bootstrap facts. CommandSource construction is wire()'s responsibility.

#### Scenario: no CommandSource in bootstrap facts

- **WHEN** any initializer returns its result
- **THEN** the result SHALL contain I/O results only, not Driver types

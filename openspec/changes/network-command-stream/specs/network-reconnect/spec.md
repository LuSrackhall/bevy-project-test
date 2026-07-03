## ADDED Requirements

### Requirement: Client detects disconnection

The client SHALL detect network disconnection when no `BroadcastFrame` has been received for 3 seconds.

#### Scenario: disconnection detected

- **WHEN** no `BroadcastFrame` arrives within 3 seconds
- **THEN** the client SHALL enter reconnecting state

### Requirement: Client sends ReconnectRequest

The client SHALL send a `ReconnectRequest` to the relay, including `game_id` and `last_tick_consumed` (the last tick the client successfully executed).

#### Scenario: reconnect request sent

- **WHEN** client has entered reconnecting state and relay is reachable
- **THEN** client SHALL send `ReconnectRequest { game_id, last_tick_consumed }`

### Requirement: Relay responds with ReconnectResponse

The relay SHALL validate the `ruleset_version` match and respond with seed, map_spec_hash, and a full command log from `last_tick_consumed + 1` to current tick.

#### Scenario: schema mismatch

- **WHEN** the client's ruleset_version does not match the relay's
- **THEN** relay SHALL return INCOMPATIBLE status, client SHALL display version mismatch error

#### Scenario: successful reconnect response

- **WHEN** ruleset_version matches
- **THEN** relay SHALL respond with `ReconnectResponse { seed, map_spec_hash, ticks: Vec<TickCommands>, players }`

### Requirement: Client rebuilds world via replay

The client SHALL reconstruct simulation state by:
1. `init_simulation_world(seed)`
2. `simulation::map::generate_map(...)`
3. Fast replay of `ticks` in sequence, using `run_tick_default()` (same function as live execution)

#### Scenario: reconnect recovers to current tick

- **WHEN** client receives `ReconnectResponse` with ticks covering tick 51–500
- **THEN** client replays all ticks and resumes normal lockstep from tick 501

### Requirement: Replay equivalence is bitwise-identical state

Replay-based reconnect SHALL guarantee bitwise-identical simulation state compared to the original execution, given identical `seed + ruleset_version + TickCommands` sequence.

#### Scenario: replay state matches live

- **WHEN** a disconnected client replays the command log from seed to current tick
- **THEN** the simulation state SHALL be bitwise-identical to the state of clients that remained connected

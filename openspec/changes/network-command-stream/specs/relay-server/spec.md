## ADDED Requirements

### Requirement: Relay Server collects player inputs per tick

The relay SHALL accept `PlayerTickFrame` messages from all connected clients. It SHALL buffer all received commands grouped by `(tick, player_id)`.

#### Scenario: collect input from one player

- **WHEN** relay receives `PlayerTickFrame { tick: 100, player_id: 1, commands: [cmd_a], .. }`
- **THEN** it SHALL store `cmd_a` under `buffer[100][1]`

#### Scenario: collect input from multiple players

- **WHEN** relay receives frames for tick 100 from players 1, 2, and 3
- **THEN** it SHALL store each player's commands independently

### Requirement: Relay finalizes tick when all players ready or timeout

The relay SHALL finalize a tick when either all expected player inputs have been collected, or the tick's timeout has elapsed.

#### Scenario: finalize when all players arrive

- **WHEN** players 1, 2, 3 have submitted frames for tick 100
- **THEN** relay SHALL finalize tick 100 immediately (without waiting for timeout)

#### Scenario: finalize on timeout with NoOp injection

- **WHEN** player 2 has not submitted frame for tick 100 within the timeout window
- **THEN** relay SHALL finalize tick 100 with `NoOp` injected for player 2

### Requirement: Relay broadcasts finalized CommandBatch

Once a tick is finalized, the relay SHALL broadcast a `CommandBatch` (wrapped in `BroadcastFrame`) to all connected clients.

- The batch SHALL contain all commands sorted by `(player_id, sort_tag)`
- The batch SHALL contain commands for ALL players (including each player's own commands — echo)
- The batch SHALL be immutable once broadcast (no late corrections)

#### Scenario: broadcast includes all players

- **WHEN** tick 100 is finalized with commands from players 1, 2 and NoOp for player 3
- **THEN** the broadcast `CommandBatch` SHALL contain commands for players 1, 2, and 3

#### Scenario: late input deferred to next tick

- **WHEN** tick 100 has already been finalized and broadcast
- **THEN** any late `PlayerTickFrame` for tick 100 SHALL be stored as input for tick 101 (or rejected)

### Requirement: Relay deduplicates by (tick, player_id, player_sid)

The relay SHALL reject duplicate `PlayerTickFrame` messages with the same `(tick, player_id, player_sid)` combination to prevent duplicate command injection.

#### Scenario: duplicate frame rejected

- **WHEN** relay receives two `PlayerTickFrame` messages with identical `(tick=100, player_id=1, player_sid=42)`
- **THEN** the second SHALL be silently dropped

### Requirement: Relay caches command log for reconnect

The relay SHALL store all finalized `CommandBatch` entries for the duration of the game session. This log SHALL be retrievable by reconnecting clients.

#### Scenario: reconnect fetches log

- **WHEN** a client sends `ReconnectRequest { game_id, last_tick_consumed: 50 }`
- **THEN** relay responds with `ReconnectResponse` containing `ticks: Vec<TickCommands>` from tick 51 to current

### Requirement: Relay does NOT simulate or modify commands

The relay SHALL NOT run simulation, assign ordering keys, modify command payloads, or inspect `Action` variants for branching decisions.

#### Scenario: relay does not inspect action payload

- **WHEN** a `PlayerTickFrame` contains commands with various `Action` variants
- **THEN** the relay SHALL forward them without inspection, modification, or conditional behavior based on action type

---

**Implementation:** `network.rs` lines 238-514 (RelayServer state machine). `relay/src/lib.rs` (TCP transport). 6 unit tests covering state machine logic. 3 TCP integration tests in `relay/tests/integration.rs`. 1 Bevy e2e test in `bevy_adapter/tests/network_e2e.rs`.

## MODIFIED Requirements

### Requirement: Relay finalizes tick when all players ready or timeout

The relay SHALL finalize a tick when either all active expected player inputs have been collected, or the tick's timeout has elapsed. A player marked `Disconnected` SHALL be treated as satisfied for the all-ready check — the tick SHALL finalize via the timeout path for that player (with NoOp injection) rather than hanging the barrier indefinitely.

#### Scenario: finalize when all players arrive

- **WHEN** players 1, 2, 3 have submitted frames for tick 100
- **THEN** relay SHALL finalize tick 100 immediately (without waiting for timeout)

#### Scenario: finalize on timeout with NoOp injection

- **WHEN** player 2 has not submitted frame for tick 100 within the timeout window
- **THEN** relay SHALL finalize tick 100 with `NoOp` injected for player 2

#### Scenario: disconnected player does not hang the barrier

- **WHEN** player 2 has disconnected and is marked `Disconnected`, and players 1, 3 have submitted frames for tick 100
- **THEN** relay SHALL finalize tick 100 (player 2 satisfied via timeout, NoOp injected) and NOT wait forever

## ADDED Requirements

### Requirement: Relay retains seats for disconnected players

The relay SHALL retain a player's seat after disconnect (mark `Disconnected`), not permanently remove them from the expected player set. On reconnect, the player SHALL be re-admitted under their original `player_id`.

#### Scenario: disconnected player keeps seat

- **WHEN** player 2 disconnects mid-game
- **THEN** the relay SHALL keep player 2's seat (expected-player set unchanged), mark them `Disconnected`, and continue finalizing ticks with NoOp for player 2

#### Scenario: lobby drop does not deadlock the room

- **WHEN** player 2 drops in the lobby (before signaling ready) and is marked `Disconnected`
- **THEN** the all-ready check SHALL exclude player 2, so the remaining players can still start the game

#### Scenario: reconnect reuses original player id

- **WHEN** the disconnected player 2 reconnects
- **THEN** the relay SHALL assign the same `player_id` 2 (not `Room is full`), remove the `Disconnected` mark, and resume accepting their frames

### Requirement: Lobby ready tracking is scalable

The relay's lobby ready tracking SHALL NOT be limited to 8 players. Ready state SHALL be tracked per-player without a bit-mask ceiling.

#### Scenario: 9 players all ready

- **WHEN** 9 players each signal `LobbyReady`
- **THEN** the relay SHALL track all 9 as ready and start the game when all are ready (no bit-mask overflow)

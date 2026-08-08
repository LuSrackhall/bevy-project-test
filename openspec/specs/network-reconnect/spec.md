# network-reconnect Specification

## Purpose
TBD - created by archiving change network-command-stream. Update Purpose after archive.
## Requirements
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

The relay SHALL validate the `ruleset_version` match and respond with seed, map_spec_hash, and the command log from `last_tick_consumed + 1` to current tick, sent over the reliable Control channel. Logs exceeding the datagram MTU are transparently fragmented by the reliable layer (IPv6 ≤1232 payload) and reassembled on the client before replay. Application-level paging (`ReconnectPage`) is a future optimization for very long disconnects.

#### Scenario: schema mismatch

- **WHEN** the client's ruleset_version does not match the relay's
- **THEN** relay SHALL return INCOMPATIBLE status, client SHALL display version mismatch error

#### Scenario: successful reconnect response (full log)

- **WHEN** ruleset_version matches
- **THEN** relay SHALL respond with `ReconnectResponse` containing the full command log (`seed`, `map_spec_hash`, `first_tick`, `ticks`); logs over MTU are fragmented by the reliable layer and reassembled before delivery

#### Scenario: full response reassembled before replay

- **WHEN** the client has reassembled the full `ReconnectResponse`
- **THEN** the client SHALL apply the command log and resume replay from `first_tick`

### Requirement: Client rebuilds world via replay

Reconnect recovery distinguishes two scenes:

- **Scene A (network drop, process alive)**: the local world is intact at the
  disconnect point — the client SHALL NOT rebuild the world, only load the missed
  ticks via `apply_reconnect` and let the driver resume. This is the implemented
  primary path in this change.
- **Scene B (process restart)**: the world MUST be rebuilt by:
  1. `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local_player_id))`
  2. `simulation::map::generate_map(...)` with the map matching `map_spec_hash`
  3. Fast replay of `ticks` in sequence, using `run_tick(enable_ai: false)` — the same entry as live network execution
  Scene B wiring is a follow-up change (not implemented here).

The client SHALL NOT use `init_simulation_world` (single-player, 2-slot) or `run_tick_default` (AI enabled) for reconnect rebuild — either would diverge from the live network path and desync.

#### Scenario: reconnect recovers to current tick (Scene A)

- **WHEN** client receives `ReconnectResponse` with ticks covering the missed range and `apply_reconnect` loads them
- **THEN** the driver resumes from the disconnect point and the missed ticks are replayed without world rebuild

#### Scenario: rebuild path matches live network init (Scene B)

- **WHEN** a disconnected client rebuilds with `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local))` + `run_tick(enable_ai:false)`
- **THEN** the resulting world is hash-identical to the live network clients' world at the same tick


### Requirement: Replay equivalence is bitwise-identical state

Replay-based reconnect SHALL guarantee bitwise-identical simulation state compared to the original execution, given identical `seed + ruleset_version + TickCommands` sequence.

#### Scenario: replay state matches live

- **WHEN** a disconnected client replays the command log from seed to current tick
- **THEN** the simulation state SHALL be bitwise-identical to the state of clients that remained connected

---

**Implementation:** `network.rs` handles reconnect on relay side (handle_reconnect). `NetworkCommandSource.apply_reconnect()` client side with ruleset_version validation. e2e test verifies deterministic replay equality.


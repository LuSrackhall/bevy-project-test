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

The relay SHALL validate the `ruleset_version` match and respond to a `ReconnectRequest` with a metadata-only `ReconnectResponse` (`game_id`, `ruleset_version`, `seed`, `map_spec_hash`, `first_tick`, `total_ticks`, `page_count`, `players`), then push `page_count` `ReconnectPage` messages on the reliable Control channel. Pages SHALL cover contiguous tick ranges bucketed by tick value (`[first_tick + i*PAGE_TICKS, first_tick + (i+1)*PAGE_TICKS)`), NOT by log position (the finalized log is append-ordered, not tick-ordered). `total_ticks` SHALL count log entries with `tick > last_tick_consumed`. Logs exceeding a page's datagram MTU are fragmented by the reliable layer. The relay SHALL NOT wait for per-page requests (push model; ReliableOrdered preserves page order).

#### Scenario: schema mismatch

- **WHEN** the client's ruleset_version does not match the relay's
- **THEN** relay SHALL return INCOMPATIBLE status, client SHALL display version mismatch error

#### Scenario: successful reconnect response is metadata plus pages

- **WHEN** ruleset_version matches and the command log from `last_tick_consumed + 1` to current tick has `N` entries
- **THEN** relay SHALL respond with `ReconnectResponse { first_tick, total_ticks: N, page_count: ceil(N/PAGE_TICKS), seed, map_spec_hash, players }` and push `page_count` `ReconnectPage` messages on the Control channel

#### Scenario: page covers a contiguous tick range by value

- **WHEN** the log was finalized out of tick order (append order ≠ tick order) and a reconnect occurs
- **THEN** each `ReconnectPage` SHALL contain exactly the log entries whose ticks fall in its bucketed range, with no gaps or duplicates across pages, and `first_tick` of page `i+1` SHALL equal the last tick of page `i` plus one

#### Scenario: empty log returns no pages

- **WHEN** `last_tick_consumed` equals the current finalized tick (no new ticks)
- **THEN** relay SHALL respond with `page_count = 0` and no `ReconnectPage` messages

#### Scenario: reconnect during frozen game is rejected

- **WHEN** the game is frozen (timeout/GameOver path)
- **THEN** relay SHALL return an error rather than a command log

### Requirement: Client applies reconnect pages progressively

The client SHALL apply each `ReconnectPage` as it is received over the reliable Control channel, inserting its `ticks` into the relay buffer so the driver can resume replay before all pages arrive. The client SHALL track a page cursor and reject out-of-order, duplicate, or out-of-range pages, and SHALL verify `page.page_count` matches the metadata `page_count` (defense against stale pages from a previous session). When `page_count = 0`, the client SHALL consider replay complete immediately without waiting for pages.

#### Scenario: pages applied progressively in order

- **WHEN** the client receives the metadata `ReconnectResponse` then pages 0..n in order
- **THEN** each page's ticks are inserted as received; ticks in page 0 become ready for replay before page n arrives; the driver advances tick-by-tick as pages fill the buffer

#### Scenario: page validation rejects stale pages

- **WHEN** a `ReconnectPage` has a `page_index` not equal to the expected next page, a duplicate `page_index`, `page_index >= page_count`, or `page_count` different from the metadata's
- **THEN** the client SHALL reject the page (ignore it) and continue waiting for the expected page

#### Scenario: replay resumes from disconnect point

- **WHEN** the client reconnects after already applying some pages (`last_tick_consumed` advanced)
- **THEN** the reconnect request resumes from the applied tick; re-applied ticks SHALL NOT duplicate gaps or overlap in the relay buffer

#### Scenario: empty page set completes replay immediately

- **WHEN** the metadata reports `total_ticks = 0` / `page_count = 0`
- **THEN** the client SHALL mark replay complete without waiting for any `ReconnectPage`

### Requirement: Client rebuilds world via replay

Reconnect recovery SHALL distinguish two scenes:

- **Scene A (network drop, process alive)**: the local world is intact at the
  disconnect point — the client SHALL NOT rebuild the world, only load the missed
  ticks via `apply_reconnect` and let the driver resume. This remains the primary
  path.
- **Scene B (process restart)**: the client SHALL rebuild the world by:
  1. `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local_player_id))`
  2. `simulation::map::generate_map(...)` with the `map_size` from the reconnect
     metadata (matching `map_spec_hash`)
  3. Fast replay of `ticks` in sequence, using `run_tick(enable_ai: false)` — the
     same entry as live network execution, in a catch-up mode that executes
     multiple ticks per frame until caught up.

The client SHALL distinguish Scene A from Scene B by driver lifecycle state: a
reconnecting client already in Playing state (world exists) SHALL take the Scene A
path without rebuilding; a fresh process (no world, in lobby) SHALL take the Scene
B path. The client SHALL NOT rebuild the world while Playing (no duplicate
rebuild). Scene B SHALL use the reconnect metadata's `seed` and `map_size` as the
single authority (verifying `GameStarted.seed` matches). During Scene B catch-up
replay the client SHALL NOT uplink local input frames (already-finalized ticks
would leak into the relay staging).

The client SHALL NOT use `init_simulation_world` (single-player, 2-slot) or `run_tick_default` (AI enabled) for reconnect rebuild — either would diverge from the live network path and desync.

#### Scenario: reconnect recovers to current tick (Scene A)

- **WHEN** client receives `ReconnectResponse` with ticks covering the missed range and `apply_reconnect` loads them
- **THEN** the driver resumes from the disconnect point and the missed ticks are replayed without world rebuild

#### Scenario: rebuild path matches live network init (Scene B)

- **WHEN** a restarted process rebuilds with `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local))` + `generate_map(map_size)` + fast replay of the full log via `run_tick(enable_ai:false)`
- **THEN** the resulting world is hash-identical to the live network clients' world at the same tick

#### Scenario: scene distinction by lifecycle state

- **WHEN** a reconnecting client is in Playing state (world exists), including a disconnect at tick 0
- **THEN** the client SHALL take the Scene A path (no world rebuild)

#### Scenario: fresh process reconnects to a started game

- **WHEN** a restarted process (in lobby, no world) reconnects to an in-progress game and receives `ReconnectResponse` with `total_ticks > 0`
- **THEN** the client SHALL rebuild the world from the metadata `seed`/`map_size` and fast-replay the full log before resuming live play

#### Scenario: no duplicate rebuild while playing

- **WHEN** a reconnecting client is already Playing and receives a second reconnect response
- **THEN** the client SHALL apply it via the Scene A path and SHALL NOT rebuild the world


### Requirement: Replay equivalence is bitwise-identical state

Replay-based reconnect SHALL guarantee bitwise-identical simulation state compared to the original execution, given identical `seed + ruleset_version + TickCommands` sequence.

#### Scenario: replay state matches live

- **WHEN** a disconnected client replays the command log from seed to current tick
- **THEN** the simulation state SHALL be bitwise-identical to the state of clients that remained connected

### Requirement: Relay sends reconnect metadata with map size

The relay SHALL include `map_size` in `ReconnectResponse`, matching the game's map configuration. The client SHALL use it for `generate_map` during Scene B rebuild so the rebuilt world matches the live clients' map (which may not be the default).

#### Scenario: non-default map rebuilds identically

- **WHEN** the game uses a non-default `MapSize` and a restarted client rebuilds
- **THEN** the relay SHALL provide that `MapSize` in the reconnect metadata and the rebuilt world SHALL match the live clients' world

---

**Implementation:** `network.rs` handles reconnect on relay side (handle_reconnect). `NetworkCommandSource.apply_reconnect()` client side with ruleset_version validation. e2e test verifies deterministic replay equality.


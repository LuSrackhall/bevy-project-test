## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: Relay sends reconnect metadata with map size

The relay SHALL include `map_size` in `ReconnectResponse`, matching the game's map configuration. The client SHALL use it for `generate_map` during Scene B rebuild so the rebuilt world matches the live clients' map (which may not be the default).

#### Scenario: non-default map rebuilds identically

- **WHEN** the game uses a non-default `MapSize` and a restarted client rebuilds
- **THEN** the relay SHALL provide that `MapSize` in the reconnect metadata and the rebuilt world SHALL match the live clients' world

## MODIFIED Requirements

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

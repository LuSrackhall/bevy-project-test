## MODIFIED Requirements

### Requirement: Client rebuilds world via replay

The client SHALL reconstruct simulation state by:
1. `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local_player_id))`
2. `simulation::map::generate_map(...)` with the map matching `map_spec_hash`
3. Fast replay of `ticks` in sequence, using `run_tick(enable_ai: false)` — the same entry as live network execution

The client SHALL NOT use `init_simulation_world` (single-player, 2-slot) or `run_tick_default` (AI enabled) for reconnect rebuild — either would diverge from the live network path and desync.

#### Scenario: reconnect recovers to current tick

- **WHEN** client receives `ReconnectResponse` with ticks covering tick 51–500
- **THEN** client replays all ticks via the network world-initialization path and resumes normal lockstep from tick 501

#### Scenario: rebuild path matches live network init

- **WHEN** a disconnected client rebuilds with `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local))` + `run_tick(enable_ai:false)`
- **THEN** the resulting world is hash-identical to the live network clients' world at the same tick

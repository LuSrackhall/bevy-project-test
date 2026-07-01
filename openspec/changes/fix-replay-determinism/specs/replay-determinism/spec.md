## ADDED Requirements

### Requirement: Replay Determinism

The replay system SHALL produce identical world state hash at every desync-check tick when replaying a recorded game with identical `(seed, map_size, commands_per_tick)`. `bevy_adapter::driver::simulation_driver_system` SHALL detect any divergence at `DESYNC_CHECK_INTERVAL` boundaries and log the tick + hash values.

#### Scenario: Replay matches live play end-to-end
- **WHEN** a game session is recorded (AI enabled, with manual player commands) from tick 0 to tick N (N >= 5000)
- **AND** the recorded `ReplayFile` is loaded and replayed from tick 0 to tick N
- **THEN** `hash_world_state` at every `DESYNC_CHECK_INTERVAL` boundary SHALL be identical between the live and replay runs

### Requirement: Driver-Level Integration Test

The `bevy_adapter::driver` test module SHALL include a test that exercises the full `SimulationDriver` pipeline (commands_for_tick → inject_commands → run_tick_default) for both Live and Replay modes, comparing hashes at each desync-check interval. This test SHALL NOT depend on bevy frame scheduling.

#### Scenario: Driver test detects determinism
- **WHEN** `test_driver_live_replay_determinism` runs with seed=42, MapSize::Small, N=5000 ticks, AI enabled, and simulated player commands
- **THEN** the test SHALL assert `hash_live_at_tick_N == hash_replay_at_tick_N` for all desync-check intervals

### Requirement: Recording Completeness

`ReplayRecorder::record_tick` SHALL record every tick's command batch, including empty batches, to ensure tick alignment between live recording and replay playback.

#### Scenario: Empty tick recorded
- **WHEN** a live game tick has zero player commands
- **THEN** `ReplayRecorder` SHALL record `(tick, vec![])` for that tick (an empty Vec)
- **AND** on replay, `commands_for_tick(tick)` SHALL return the same empty Vec

### Requirement: Diagnostic Hash Frequency Control

During diagnostic mode, DESYNC check frequency SHALL be configurable at compile time or via feature flag. The default SHALL remain 20 ticks.

#### Scenario: Per-tick hash in diagnostic mode
- **WHEN** replay determinism diagnosis is active (< 100% confident on root cause)
- **THEN** the driver SHALL support checking hash at every tick (interval=1) to pinpoint the exact divergence tick

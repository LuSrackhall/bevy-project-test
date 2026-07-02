## ADDED Requirements

### Requirement: Replay Determinism

The replay system SHALL produce identical world state hash at every desync-check tick when replaying a recorded game with identical `(seed, map_size, commands_per_tick)`. `bevy_adapter::driver::simulation_driver_system` SHALL detect any divergence at `DESYNC_CHECK_INTERVAL` boundaries and log the tick + hash values.

#### Scenario: Replay matches live play end-to-end
- **WHEN** a game session is recorded (AI enabled, with manual player commands) from tick 0 to tick N (N >= 5000)
- **AND** the recorded `ReplayFile` is loaded and replayed from tick 0 to tick N
- **THEN** `hash_world_state` at every `DESYNC_CHECK_INTERVAL` boundary SHALL be identical between the live and replay runs

### Requirement: Command Recording Completeness

All player-initiated actions that modify simulation state SHALL be recorded as `GameCommand` in the command buffer. Direct simulation state modification without a corresponding `GameCommand` SHALL NOT occur in code paths accessible during live gameplay.

#### Scenario: Spawn type change recorded
- **WHEN** a player clicks a spawn type button during live play
- **THEN** a `GameCommand { action: Action::SetSpawnType { city, soldier_type } }` SHALL be pushed to `CommandBuffer`
- **AND** on replay, the same `SetSpawnType` command SHALL be injected and have the same effect on `CityComponent.spawn_type`

### Requirement: Replay End-of-File Handling

The replay SHALL pause automatically when `driver.clock.current_tick` reaches or exceeds `ReplayFile.total_ticks`. The seek SHALL NOT allow seeking beyond `total_ticks`.

#### Scenario: Replay pauses at end
- **WHEN** replay reaches `total_ticks`
- **THEN** `driver.scheduler.is_paused` SHALL be `true`

### Requirement: Driver-Level Integration Test

The `bevy_adapter::driver` test module SHALL include a test that exercises the full `SimulationDriver` pipeline (commands_for_tick → inject_commands → run_tick_default) for both Live and Replay modes, comparing hashes at each desync-check interval. This test SHALL NOT depend on bevy frame scheduling.

#### Scenario: Driver test detects determinism
- **WHEN** `test_driver_live_replay_determinism` runs with seed=42, MapSize::Small, N=15000 ticks, AI enabled, and simulated player commands across multiple tick ranges
- **THEN** the test SHALL assert `hash_live_at_tick_N == hash_replay_at_tick_N` for all desync-check intervals

### Requirement: Recording Completeness

`ReplayRecorder::record_tick` SHALL record every tick's command batch, including empty batches, to ensure tick alignment between live recording and replay playback.

#### Scenario: Empty tick recorded
- **WHEN** a live game tick has zero player commands
- **THEN** `ReplayRecorder` SHALL record `(tick, vec![])` for that tick (an empty Vec)
- **AND** on replay, `commands_for_tick(tick)` SHALL return the same empty Vec

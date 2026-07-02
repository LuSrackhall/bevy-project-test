## ADDED Requirements

### Requirement: Single Mutation Entry

All external state modification requests targeting the simulation world SHALL be expressed as `GameCommand` and pass through `CommandSink::submit_command()` → `CommandScheduler` → `Scheduled` → `Simulation::run_tick`. Direct mutation of `simulation::World` from outside the `simulation` and `bevy_adapter` crates is FORBIDDEN.

#### Scenario: Observer cannot mutate world
- **WHEN** a render_view observer receives a UI event during live gameplay
- **THEN** the observer SHALL NOT directly modify any component or resource in `simulation::World`
- **AND** the only permitted side effect is pushing `GameCommand` to `CommandBuffer`

### Requirement: SimulationReader + CommandSink Separation

Read access and write access to the simulation world from external crates SHALL be separated into two distinct traits: `SimulationReader` (read-only structural query) and `CommandSink` (command submission).

#### Scenario: RenderView read-only access
- **WHEN** a render_view system needs to query simulation state (e.g., `update_top_bar`)
- **THEN** it SHALL inject `Res<impl SimulationReader>` as a system parameter
- **AND** it SHALL NOT have access to `&mut simulation::World`

#### Scenario: Button push recorded command
- **WHEN** a UI button observer needs to effect a simulation state change
- **THEN** it SHALL inject `ResMut<impl CommandSink>` and call `submit_command()`
- **AND** the command SHALL be recorded by `ReplayRecorder` for replay determinism

### Requirement: CommandSink Pure Transport

`CommandSink::submit_command()` SHALL be a pure transport interface with zero semantic branching. It MUST NOT inspect or branch on `GameCommand` content (action type, unit IDs, faction, etc.). All semantic processing (validation, auth, dedup, priority) SHALL occur in `CommandScheduler`.

### Requirement: CommandSource Polymorphism

All external command producers (PlayerInput, AI, NetworkReceiver, ReplayCommandSource, ScenarioRunner) SHALL implement the `CommandSource` trait. The driver SHALL NOT inspect the concrete type of `CommandSource` at runtime. The trait SHALL NOT include `is_replay()` or any source-type-identifying method.

#### Scenario: Driver agnostic to source
- **WHEN** `simulation_driver_system` processes ticks
- **THEN** it SHALL call `source.commands_for_tick(tick, ctx)` without checking whether the source is Live, Replay, Network, or AI
- **AND** `source.total_ticks()` SHALL be the only source-characterizing method (returns `Some(N)` for finite sources, `None` for streaming)

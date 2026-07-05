## ADDED Requirements

### Requirement: BootstrapPhase SHALL be the single lifecycle gate

`SimulationDriver` SHALL contain a `bootstrap_phase: BootstrapPhase` field:

```rust
pub enum BootstrapPhase {
    Init,   // Bootstrap may enter
    Wired,  // wire() completed, resources deferred (impl detail)
    Active, // Tick loop may run
}
```

- `bootstrap()` SHALL check `phase == Init` before proceeding
- `wire()` SHALL set `phase = Wired` after writing all resources
- `simulation_driver_system` SHALL check `phase == Active` before advancing ticks

#### Scenario: tick not advanced before Active

- **WHEN** `phase == Init` or `phase == Wired`
- **THEN** `simulation_driver_system` SHALL NOT advance any simulation ticks

### Requirement: SessionArtifacts is move-only, consumed by wire

`SessionArtifacts` SHALL be an enum, passed by move to `wire()`. After `wire()` returns, the artifact is dropped.

#### Scenario: artifacts cannot be cloned

- **WHEN** any code attempts to clone or retain `SessionArtifacts`
- **THEN** the compiler SHALL reject it (no `Clone` derive, no `Arc` wrapping)

### Requirement: commit order is fixed

wire() SHALL commit changes in this fixed order:
1. `init_world()`
2. `setup_recorder()`
3. `insert_resource()` for transport resources
4. `driver.source = ...`
5. `driver.phase = Wired`

#### Scenario: driver.source is last mutation

- **WHEN** wire() is executing
- **THEN** `driver.source` SHALL be assigned after all other system mutations are complete

## ADDED Requirements

### Requirement: independent benchmark binary crate

A `crates/bench/` binary crate SHALL be created depending on the simulation library. Per constitution §21, benchmark SHALL be an independent binary, not a feature flag inside simulation.

#### Scenario: bench crate compilation
- **WHEN** `cargo bench -p bench` is executed
- **THEN** criterion benchmarks SHALL run against simulation functions

#### Scenario: simulation has no benchmark feature
- **WHEN** simulation Cargo.toml is inspected
- **THEN** it SHALL NOT contain a `benchmark` feature flag

### Requirement: criterion benchmarks per phase

Benchmarks SHALL cover: full run_tick, and each individual simulation phase. Scenarios SHALL include: empty world, 1k idle soldiers, 1k vs 1k combat, 10k idle soldiers.

#### Scenario: full tick benchmark
- **WHEN** the full tick benchmark runs with 1k combat scenario
- **THEN** it SHALL measure the total time of run_tick(&mut world, tick, &config)

#### Scenario: per-phase benchmark
- **WHEN** the combat_engagement_system benchmark runs
- **THEN** it SHALL measure only that system's execution time

### Requirement: CI performance regression gate

CI SHALL run `cargo bench` and compare against a saved baseline. A regression exceeding 5% SHALL fail the build.

#### Scenario: no regression
- **WHEN** a PR does not change any hot-path code
- **THEN** CI bench SHALL pass

#### Scenario: regression detected
- **WHEN** a PR causes a 6% regression in a benchmark
- **THEN** CI SHALL fail with a message indicating which benchmark regressed

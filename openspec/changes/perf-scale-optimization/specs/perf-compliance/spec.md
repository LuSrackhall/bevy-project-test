## ADDED Requirements

### Requirement: §4.3 complexity declarations on all hot systems

Every simulation system marked as Hot Path (Yes) SHALL have structured doc-comments declaring Complexity, Memory, and Hot Path fields.

#### Scenario: combat system declarations
- **WHEN** combat_engagement_system source code is read
- **THEN** it SHALL contain doc-comments with `Complexity: O(s*k)`, `Memory: O(s)`, `Hot Path: Yes`

#### Scenario: movement system declarations
- **WHEN** soldier_movement_system source code is read
- **THEN** it SHALL contain structured complexity doc-comments

### Requirement: docs/performance.md

A `docs/performance.md` file SHALL be created documenting: performance baselines (tick time at 1k/5k/10k units), optimization history, scaling thresholds (5k/10k/50k/100k/1M), and measurement methodology.

#### Scenario: performance doc exists
- **WHEN** docs/performance.md is read
- **THEN** it SHALL contain baseline measurements and scaling threshold documentation

### Requirement: ADR-004 and ADR-005

ADR-004 SHALL document the decision to maintain SpatialHash as per-tick construction vs persistent incremental index. ADR-005 SHALL document the phase dependency graph and future parallelism strategy.

#### Scenario: ADR-004 content
- **WHEN** docs/adr/0004-spatial-hash-lifecycle.md is read
- **THEN** it SHALL document the per-tick construction decision with alternatives and modification conditions

#### Scenario: ADR-005 content
- **WHEN** docs/adr/0005-phase-dependency-graph.md is read
- **THEN** it SHALL document which phases are independent and which have data dependencies

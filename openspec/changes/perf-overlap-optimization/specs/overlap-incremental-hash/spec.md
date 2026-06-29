## ADDED Requirements

### Requirement: incremental SpatialHash update between iterations

overlap_resolution_system SHALL NOT rebuild SpatialHash from scratch each iteration. Instead, it SHALL update only displaced units: remove from old cell, insert into new cell.

#### Scenario: no displacements
- **WHEN** an iteration produces zero displacements
- **THEN** SpatialHash SHALL NOT be rebuilt

#### Scenario: some displacements
- **WHEN** an iteration displaces 50 units
- **THEN** only those 50 units SHALL be updated in SpatialHash (not all 1000)

#### Scenario: determinism preserved
- **WHEN** incremental update is used
- **THEN** SpatialHash cell traversal order SHALL remain deterministic (BTreeMap + UnitId sort)

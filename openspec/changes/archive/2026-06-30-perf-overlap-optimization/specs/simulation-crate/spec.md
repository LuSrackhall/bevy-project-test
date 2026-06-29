## MODIFIED Requirements

### Requirement: overlap_resolution_system performance

overlap_resolution_system SHALL use squared distance early-out, incremental SpatialHash, and adaptive iteration exit. At 1000 packed units, execution time SHALL be under 10ms (down from 28.2ms).

#### Scenario: performance at 1000 units
- **WHEN** 1000 units are densely packed
- **THEN** overlap_resolution_system SHALL complete in under 10ms

#### Scenario: golden test determinism
- **WHEN** golden_test runs with the optimized system
- **THEN** world state hash SHALL match the pre-optimization baseline

## ADDED Requirements

### Requirement: query_range generic interface

SpatialHash SHALL provide a `query_range(pos: FixedVec2, radius: i64)` method that sweeps all cells within the given radius. The number of cells swept SHALL be `(2 * ceil(radius / cell_size) + 1)^2`. Cell iteration order SHALL be deterministic (BTreeMap key order).

#### Scenario: small radius query
- **WHEN** query_range is called with radius=30 and cell_size=32
- **THEN** it SHALL sweep 9 cells (3×3 neighborhood), identical to current query_nearby

#### Scenario: large radius query
- **WHEN** query_range is called with radius=200 and cell_size=64
- **THEN** it SHALL sweep 49 cells (7×7 neighborhood)

#### Scenario: deterministic iteration
- **WHEN** query_range is called twice with identical parameters
- **THEN** entries SHALL be returned in the same order both times

### Requirement: per-system cell_size preservation

Each combat system SHALL continue to use its own cell_size when building SpatialHash. Systems SHALL NOT be forced to use a unified cell_size.

#### Scenario: melee system cell_size
- **WHEN** melee_attack_system builds SpatialHash
- **THEN** cell_size SHALL be 32

#### Scenario: archer system cell_size
- **WHEN** archer_attack_system builds SpatialHash
- **THEN** cell_size SHALL be 200

### Requirement: query_nearby backward compatibility

The existing query_nearby method SHALL remain available. query_range is an addition, not a replacement.

#### Scenario: existing code unaffected
- **WHEN** systems call query_nearby with cell_size that makes 3×3 sufficient
- **THEN** behavior SHALL be identical to before this change

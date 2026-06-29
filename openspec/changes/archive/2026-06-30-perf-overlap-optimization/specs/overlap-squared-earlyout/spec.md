## ADDED Requirements

### Requirement: squared distance early-out before integer_sqrt

overlap_resolution_system SHALL compare `dist_sq.0` against `min_dist_sq` before calling `integer_sqrt`. Only pairs where `dist_sq.0 < min_dist_sq` SHALL proceed to sqrt computation.

#### Scenario: non-overlapping pair skipped
- **WHEN** two units are farther apart than their combined collision radii
- **THEN** `integer_sqrt` SHALL NOT be called for that pair

#### Scenario: overlapping pair processed
- **WHEN** two units are closer than their combined collision radii
- **THEN** `integer_sqrt` SHALL be called and push vector computed normally

#### Scenario: determinism preserved
- **WHEN** two runs with identical inputs execute overlap_resolution
- **THEN** the same displacements SHALL be produced

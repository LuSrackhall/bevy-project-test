## ADDED Requirements

### Requirement: per-faction SpatialHash

TickCombatIndex SHALL contain per-faction SpatialHash indices. Combat systems SHALL query only enemy faction hashes, skipping friendly units.

#### Scenario: faction filter
- **WHEN** melee_attack queries neighbors for a Player soldier
- **THEN** only Enemy faction SpatialHash cells SHALL be scanned

#### Scenario: determinism
- **WHEN** same input executes combat
- **THEN** per-faction SpatialHash traversal order SHALL be deterministic (BTreeMap + UnitId sort)

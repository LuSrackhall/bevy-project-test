## MODIFIED Requirements

### Requirement: combat_engagement_system uses HashMap lookup

combat_engagement_system SHALL collect soldiers into a HashMap<UnitId, SoldierData> for O(1) lookup. The outer iteration over sorted_soldier_uids SHALL maintain deterministic ordering. This eliminates the O(S²) Vec::find pattern.

#### Scenario: performance at scale
- **WHEN** 1000 soldiers engage in combat
- **THEN** combat_engagement_system SHALL complete in O(S*k) time, not O(S²)

#### Scenario: determinism preserved
- **WHEN** two identical runs execute combat_engagement_system
- **THEN** target selection and attack order SHALL be identical

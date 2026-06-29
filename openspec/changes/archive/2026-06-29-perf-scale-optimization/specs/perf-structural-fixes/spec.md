## ADDED Requirements

### Requirement: combat_engagement O(S²) elimination

combat_engagement_system SHALL use HashMap<UnitId, ...> for soldier data lookup instead of Vec::find linear scan. The outer loop over sorted_soldier_uids SHALL maintain deterministic ordering. Inner lookup SHALL use HashMap::get() which does not depend on iteration order.

#### Scenario: engagement target selection determinism
- **WHEN** two runs with identical inputs and seed execute combat_engagement_system
- **THEN** the same targets SHALL be selected in the same order, producing identical SimulationEvents

#### Scenario: engagement performance at 1000 units
- **WHEN** 1000 soldiers are engaged in combat
- **THEN** combat_engagement_system SHALL NOT contain any O(S²) operations

### Requirement: build_soldier_index helper function

A `build_soldier_index(world: &mut World) -> HashMap<UnitId, SoldierSnapshot>` function SHALL be provided. Each combat system SHALL call this function independently per invocation. Systems SHALL NOT share a single HashMap instance across phases because entity lifetimes change between phases.

#### Scenario: helper called independently per system
- **WHEN** melee_attack_system and arrow_movement_system both need soldier data
- **THEN** each SHALL call build_soldier_index independently, not share a Resource

#### Scenario: stale entity safety
- **WHEN** melee_attack_system kills an entity and arrow_movement_system later queries soldier data
- **THEN** arrow_movement_system's independent build SHALL NOT contain the killed entity

### Requirement: overlap_resolution SpatialHash reuse

overlap_resolution_system SHALL build SpatialHash once before the iteration loop and reuse it across iterations. SpatialHash SHALL only be rebuilt if positions actually changed during an iteration.

#### Scenario: overlap with convergent positions
- **WHEN** overlap_resolution runs with units that have no overlap
- **THEN** SpatialHash SHALL be built once and reused for all iterations without rebuild

#### Scenario: overlap with position changes
- **WHEN** an iteration moves units to resolve overlap
- **THEN** SpatialHash SHALL be rebuilt before the next iteration

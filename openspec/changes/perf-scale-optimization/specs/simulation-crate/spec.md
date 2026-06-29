## MODIFIED Requirements

### Requirement: UnitIdEntityIndex Resource

UnitIdEntityIndex SHALL be a Resource in the simulation World. It SHALL be maintained incrementally: insert on spawn, remove on despawn. It SHALL NOT be rebuilt from scratch each tick.

#### Scenario: index available at tick start
- **WHEN** run_tick begins
- **THEN** UnitIdEntityIndex SHALL be available as a Resource containing all currently alive entities

#### Scenario: index updated during tick
- **WHEN** an entity is spawned or despawned during a tick
- **THEN** UnitIdEntityIndex SHALL reflect the change immediately (not deferred to next tick)

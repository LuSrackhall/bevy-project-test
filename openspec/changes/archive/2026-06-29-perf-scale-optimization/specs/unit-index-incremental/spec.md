## ADDED Requirements

### Requirement: incremental UnitIdEntityIndex updates

UnitIdEntityIndex SHALL be maintained incrementally via insert on spawn and remove on despawn. The per-tick full rebuild from World query SHALL be removed.

#### Scenario: spawn inserts into index
- **WHEN** a new soldier entity is spawned via consume_commands_system
- **THEN** its (UnitId, Entity) pair SHALL be inserted into UnitIdEntityIndex

#### Scenario: despawn removes from index
- **WHEN** a soldier entity is despawned (killed in combat, captured, etc.)
- **THEN** its UnitId SHALL be removed from UnitIdEntityIndex

### Requirement: stale entity safety net

find_entity_by_unit_id SHALL continue to verify Entity validity via `world.get_entity(entity).is_ok()` even after incremental index changes.

#### Scenario: despawned entity lookup
- **WHEN** find_entity_by_unit_id is called for a UnitId whose entity was despawned this tick
- **THEN** it SHALL return None (not a stale Entity)

### Requirement: UnitIdMapper independence

UnitIdMapper in bevy_adapter SHALL remain a separate index from UnitIdEntityIndex in simulation. They SHALL NOT be merged because simulation cannot depend on bevy_adapter (layer topology).

#### Scenario: separate index maintenance
- **WHEN** UnitIdEntityIndex is updated in simulation
- **THEN** UnitIdMapper in bevy_adapter SHALL be updated independently through its own mechanism

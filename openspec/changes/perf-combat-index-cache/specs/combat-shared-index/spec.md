## ADDED Requirements

### Requirement: shared TickCombatIndex

A TickCombatIndex Resource SHALL be built once per tick before combat phases. All combat systems SHALL read from this shared index instead of independently rebuilding identical data structures.

#### Scenario: single build per tick
- **WHEN** run_tick begins
- **THEN** TickCombatIndex SHALL be built once and inserted as a World Resource

#### Scenario: combat systems use shared index
- **WHEN** any combat system needs soldier data (positions, factions)
- **THEN** it SHALL read from TickCombatIndex Resource, not rebuild from World queries

#### Scenario: stale entity safety
- **WHEN** a combat system reads an Entity from TickCombatIndex for a unit killed earlier in the tick
- **THEN** find_entity_by_unit_id SHALL return None (via world.get_entity().is_ok() check)

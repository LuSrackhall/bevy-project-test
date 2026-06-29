## ADDED Requirements

### Requirement: InfoBar dirty tracking

unit_info_bar_system SHALL cache HP/Level/EXP/Shield values per unit. format! + Text2d mutation SHALL only execute when cached value differs from current.

#### Scenario: unchanged values skipped
- **WHEN** a unit's HP, Level, EXP, and Shield are unchanged since last frame
- **THEN** format! and Text2d update SHALL NOT execute for that unit

#### Scenario: changed values updated
- **WHEN** a unit takes damage (HP changes)
- **THEN** format! and Text2d update SHALL execute and cache SHALL be updated

#### Scenario: new unit
- **WHEN** a unit has no cache entry
- **THEN** update SHALL execute unconditionally and cache SHALL be inserted

#### Scenario: dead unit cleanup
- **WHEN** a unit is detected as dead via existing dead_ids logic
- **THEN** its cache entry SHALL be removed

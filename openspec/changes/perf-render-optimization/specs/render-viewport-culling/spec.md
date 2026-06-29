## ADDED Requirements

### Requirement: viewport culling for render systems

unit_info_bar_system and draw_debug_shapes_system SHALL skip units outside the camera viewport AABB.

#### Scenario: off-screen unit skipped
- **WHEN** a unit's LogicalPosition is outside the camera viewport AABB
- **THEN** Gizmos draw and InfoBar update SHALL NOT execute for that unit

#### Scenario: on-screen unit rendered
- **WHEN** a unit's LogicalPosition is inside the camera viewport AABB
- **THEN** Gizmos draw and InfoBar update SHALL execute normally

#### Scenario: all units visible
- **WHEN** the viewport AABB contains the entire map bounds
- **THEN** culling SHALL be skipped entirely (no per-unit AABB checks)

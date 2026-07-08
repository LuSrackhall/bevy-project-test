# pvp-debug-colors Specification

## Purpose
TBD - created by archiving change pvp-visual-polish. Update Purpose after archive.
## Requirements
### Requirement: Debug colors use LocalPlayerId

The `draw_debug_shapes_system` SHALL use `LocalPlayerId` to determine which faction receives "player" (blue) coloring and which receives "enemy" (red) coloring.

- Player's own faction → blue-tinted
- Active enemy factions (FactionId(0) or FactionId(1) not owned by local player) → red-tinted
- Non-active factions (FactionId(2+)) → gray

#### Scenario: Single-player debug colors unchanged
- **WHEN** a single-player game is running
- **THEN** `lid=0`, Player FactionId(0) is blue, AI FactionId(1) is red

#### Scenario: PvP Player 2 debug colors correct
- **WHEN** Player 2 (`LocalPlayerId(1)`) runs with debug rendering
- **THEN** `lid=1`, Player 2's FactionId(1) units → blue, Player 1's FactionId(0) units → red


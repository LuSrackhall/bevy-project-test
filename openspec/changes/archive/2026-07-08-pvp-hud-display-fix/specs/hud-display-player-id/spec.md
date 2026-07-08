## ADDED Requirements

### Requirement: HUD display uses LocalPlayerId

The `update_top_bar` and `seek_panel_count_system` functions SHALL use `LocalPlayerId` from the simulation world to determine which faction data to display, rather than hardcoding `FactionId(0)`.

- The player_id SHALL be obtained via `crate::local_player_id(&*sim_world)` where `sim_world` is already a system parameter
- If `LocalPlayerId` is absent (single-player default), the system SHALL fall back to `0`

#### Scenario: Single-player top bar shows correct data
- **WHEN** a single-player game is running
- **THEN** `update_top_bar` SHALL display population and soldier counts for `FactionId(0)` (same as before)

#### Scenario: Multiplayer top bar shows local player data
- **WHEN** a networked game is running and the local player has `LocalPlayerId(n)`
- **THEN** `update_top_bar` SHALL display population and soldier counts for `FactionId(n)`

#### Scenario: Seek panel counts correct faction
- **WHEN** the seek panel is shown in a multiplayer game where the local player has `LocalPlayerId(n)`
- **THEN** `seek_panel_count_system` SHALL count soldiers belonging to `FactionId(n)`

### Requirement: Faction label dispatch

The faction label match arm SHALL use a guard pattern to label the local player's faction as "玩家", and all other factions as "其他".

#### Scenario: Local player labeled correctly
- **WHEN** the top bar displays a faction matching `FactionId(lid)`
- **THEN** it SHALL show the label "玩家"

#### Scenario: Non-local factions labeled generically
- **WHEN** the top bar displays any faction not matching `FactionId(lid)`
- **THEN** it SHALL show the label "其他"
- **AND** this SHALL apply regardless of the faction ID value

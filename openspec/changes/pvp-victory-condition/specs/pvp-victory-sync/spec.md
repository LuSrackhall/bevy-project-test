## ADDED Requirements

### Requirement: Victory condition uses LocalPlayerId

The `check_victory_system` SHALL use `LocalPlayerId` from the simulation world to determine which faction is the local player's.

- The player_id SHALL be obtained via `crate::local_player_id(&*sim_world)`
- If `LocalPlayerId` is absent (single-player default), SHALL fall back to `0`

#### Scenario: Single-player victory still works
- **WHEN** a single-player game is running and the AI (FactionId(1)) is eliminated
- **THEN** `has_enemy` SHALL be false
- **AND** the system SHALL set `GameState::GameOver`

#### Scenario: PvP Player 2 victory
- **WHEN** Player 2 (`LocalPlayerId(1)`) eliminates Player 1 (`FactionId(0)`)
- **THEN** `has_enemy` SHALL be false (Player 1 eliminated)
- **AND** the system SHALL set `GameState::GameOver`

### Requirement: Neutral factions excluded from victory check

The victory check SHALL only consider active player factions (from `PlayerSlots`). Non-active factions such as neutral cities (FactionId(2)) SHALL be excluded.

#### Scenario: Neutral cities don't prevent game over
- **WHEN** all active enemy factions have been eliminated but neutral cities still exist
- **THEN** `has_enemy` SHALL be false despite neutral cities on the map
- **AND** the system SHALL set `GameState::GameOver`

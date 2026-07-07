# hud-player-id Specification

## Purpose
TBD - created by archiving change pvp-hud-command-fix. Update Purpose after archive.
## Requirements
### Requirement: HUD commands use LocalPlayerId

All HUD button observers in `render_view` that submit `GameCommand` SHALL use the local player's `player_id` from the simulation world's `LocalPlayerId` resource.

- The player_id SHALL be obtained via `simulation::types::LocalPlayerId` resource in the simulation world
- If `LocalPlayerId` is absent (single-player default), the system SHALL fall back to `0`
- The fallback value `0` SHALL be equivalent to the previous hardcoded value in single-player mode

#### Scenario: Single-player HUD command still works
- **WHEN** a single-player game is running and the player clicks any HUD button (spawn soldier, toggle shield, issue seek stance)
- **THEN** the observer SHALL read `LocalPlayerId` from the simulation world
- **AND** since `LocalPlayerId` is absent in single-player mode, SHALL fall back to `player_id: 0`
- **AND** the resulting `GameCommand` SHALL be identical to the previous hardcoded behavior

#### Scenario: Multiplayer HUD command uses correct player_id
- **WHEN** a networked game is running and the local player has `LocalPlayerId(n)` in the simulation world (where `n > 0`)
- **THEN** the observer SHALL read `LocalPlayerId(n)` from the simulation world
- **AND** the resulting `GameCommand` SHALL use `player_id: n`

### Requirement: LocalPlayerId helper function

The `render_view` crate SHALL provide a `pub(crate)` helper function that encapsulates the `LocalPlayerId` lookup + fallback pattern.

- The function SHALL accept `&SimulationWorld` (not the main Bevy `World`) as its parameter
- The signature SHALL be `pub(crate) fn local_player_id(sim: &bevy_adapter::tick::SimulationWorld) -> u8`
- The existing private implementation in `selection.rs` and the inline implementation in `camera.rs` SHALL be migrated to use this function

#### Scenario: Existing consumers migrate to shared function
- **WHEN** `selection.rs` and `camera.rs` need to read the local player ID
- **THEN** they SHALL call the public helper function from `render_view/src/lib.rs`
- **AND** the behavior SHALL remain identical (same fallback to `0` when `LocalPlayerId` is absent)


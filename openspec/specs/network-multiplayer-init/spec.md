# network-multiplayer-init Specification

## Purpose
TBD - created by archiving change network-multiplayer-init. Update Purpose after archive.
## Requirements
### Requirement: Network mode uses dynamic PlayerSlots

#### Scenario: 3-player network game
- **WHEN** a network game starts with player_count=3
- **THEN** the simulation SHALL use PlayerSlots::multi_player(3, player_id)


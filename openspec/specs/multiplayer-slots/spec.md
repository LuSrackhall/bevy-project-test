# multiplayer-slots Specification

## Purpose
TBD - created by archiving change simulation-multiplayer-slots. Update Purpose after archive.
## Requirements
### Requirement: Multi-player slot generation

PlayerSlots SHALL support generating N Human slot configurations for any N up to the `u8` type limit (255). No 8-player ceiling SHALL apply.

#### Scenario: 3-player FFA
- **WHEN** `PlayerSlots::multi_player(3, 0)` is called
- **THEN** it SHALL return 3 slots with FactionId(0), FactionId(1), FactionId(2)

#### Scenario: 9-player FFA exceeds former ceiling
- **WHEN** `PlayerSlots::multi_player(9, 3)` is called
- **THEN** it SHALL return 9 slots (FactionId 0..=8) without panicking on an 8-player assertion

#### Scenario: 16-player FFA
- **WHEN** `PlayerSlots::multi_player(16, 5)` is called
- **THEN** it SHALL return 16 slots with FactionId 0..=15, local slot 5 marked `HumanLocal`


### Requirement: Backward compatible init

`init_simulation_world(seed)` SHALL use single_player() as default.


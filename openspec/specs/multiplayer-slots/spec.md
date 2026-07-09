# multiplayer-slots Specification

## Purpose
TBD - created by archiving change simulation-multiplayer-slots. Update Purpose after archive.
## Requirements
### Requirement: Multi-player slot generation

PlayerSlots SHALL support generating N Human slot configurations.

#### Scenario: 3-player FFA
- **WHEN** `PlayerSlots::multi_player(3, 0)` is called
- **THEN** it SHALL return 3 slots with FactionId(0), FactionId(1), FactionId(2)

### Requirement: Backward compatible init

`init_simulation_world(seed)` SHALL use single_player() as default.


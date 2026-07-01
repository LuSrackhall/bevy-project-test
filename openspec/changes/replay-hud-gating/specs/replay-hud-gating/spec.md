## ADDED Requirements

### Requirement: Command Observer Replay Gate

All HUD observer callbacks that inject commands SHALL check `GameMode::Replay` at entry and return immediately without side effects during replay.

#### Scenario: Toolbar button during replay
- **WHEN** a toolbar button (shield, force-move) is activated in replay mode
- **THEN** the observer SHALL return without pushing any command to `CommandBuffer`

#### Scenario: Seek issue button during replay
- **WHEN** the seek issue ("下发") button is activated in replay mode
- **THEN** the observer SHALL return without pushing `SetSeekStance` to `CommandBuffer`

#### Scenario: Spawn type button during replay
- **WHEN** a spawn type button is activated in replay mode
- **THEN** the observer SHALL return without pushing `SetSpawnType` to `CommandBuffer`

### Requirement: Toolbar and Seek Panel Hidden in Replay

The HUD toolbar and seek panel interactive areas SHALL be hidden (`Visibility::Hidden`) during replay mode.

#### Scenario: Toolbar hidden on replay enter
- **WHEN** entering Playing state with `GameMode::Replay`
- **THEN** the toolbar container SHALL have `Visibility::Hidden` within the first Update frame

#### Scenario: Seek panel hidden on replay enter
- **WHEN** entering Playing state with `GameMode::Replay`
- **THEN** the seek panel root SHALL have `Visibility::Hidden` within the first Update frame

### Requirement: Top Bar Updates During Replay

`update_top_bar` SHALL continue running during replay mode, displaying real-time faction counts using `world_stats::count_factions`.

#### Scenario: Top bar reflects replay state
- **WHEN** a replay is playing and simulation state changes
- **THEN** the HUD top bar SHALL update soldier/city counts to reflect the current tick's world state

### Requirement: HUD Update Systems Gated in Replay

All HUD Update systems except `update_top_bar` SHALL be gated behind `not(GameMode::Replay)` to avoid unnecessary per-frame O(N) entity scans during replay.

#### Scenario: Bottom panel skipped during replay
- **WHEN** a replay is playing
- **THEN** `update_bottom_panel` SHALL NOT execute

#### Scenario: Seek panel systems skipped during replay
- **WHEN** a replay is playing
- **THEN** `seek_panel_count_system` and `seek_panel_input_system` SHALL NOT execute

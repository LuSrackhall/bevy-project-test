## ADDED Requirements

### Requirement: Main menu SHALL have Network start entry

The main menu SHALL display a Network section offering:
- Relay address text input
- Player count selector (2-8)
- "Start" button that emits `GameIntent::Network { relay_addr, player_count }`

#### Scenario: user starts network game

- **WHEN** user enters relay address and clicks Start
- **THEN** `GameIntent::Network { relay_addr, player_count }` is emitted and bootstrap begins

#### Scenario: default values

- **WHEN** Network panel opens
- **THEN** relay address defaults to `127.0.0.1:9876` and player count defaults to `2`

### Requirement: UI SHALL display connecting state

Before bootstrap completes, the UI SHALL display a "SessionConnecting" state to indicate network handshake is in progress.

#### Scenario: connecting state shown

- **WHEN** user clicks Start but bootstrap has not completed
- **THEN** a "Connecting..." message is displayed (not a frozen UI)

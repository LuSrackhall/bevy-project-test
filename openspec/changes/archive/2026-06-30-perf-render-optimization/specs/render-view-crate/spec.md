## MODIFIED Requirements

### Requirement: InfoBarMode default

The default InfoBarMode SHALL be `Selected` instead of `Classic`. Users SHALL still be able to toggle to Classic via Ctrl+H.

#### Scenario: default mode
- **WHEN** the game starts with no user configuration
- **THEN** InfoBarMode SHALL be Selected

#### Scenario: mode toggle
- **WHEN** user presses Ctrl+H
- **THEN** InfoBarMode SHALL cycle between Selected and Classic

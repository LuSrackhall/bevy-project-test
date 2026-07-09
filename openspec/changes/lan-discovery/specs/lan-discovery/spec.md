## ADDED Requirements

### Requirement: UDP beacon broadcast

The relay SHALL broadcast UDP beacons on the LAN at regular intervals.

#### Scenario: Relay discovered on LAN
- **WHEN** a relay starts
- **THEN** it SHALL broadcast UDP beacons to `255.255.255.255` every 3 seconds

### Requirement: Client discovery and display

The client SHALL listen for UDP beacons on the main menu and display discovered servers.

#### Scenario: Server appears in list
- **WHEN** a UDP beacon is received
- **THEN** the server SHALL appear in the LAN server list
- **AND** clicking it SHALL start the game connection

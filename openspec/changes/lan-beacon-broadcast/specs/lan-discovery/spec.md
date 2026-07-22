## MODIFIED Requirements

### Requirement: UDP beacon broadcast

The relay SHALL broadcast UDP beacons on the LAN at regular intervals. This applies to ALL relay implementations, including `ThreadRelayRuntime` (running as a background thread) and standalone `relay` binary.

The UDP socket MUST be bound to `0.0.0.0:{tcp_port}` (not `127.0.0.1`) and MUST have `set_broadcast(true)` enabled.

Beacon errors MUST NOT prevent the relay from starting or operating. All UDP-related errors (bind, encode, send) SHALL be logged but silently ignored for relay operation.

#### Scenario: Beacon broadcast on TCP listener start
- **WHEN** a relay starts (including `ThreadRelayRuntime`)
- **THEN** it SHALL broadcast a `LanDiscoveryPacket` to `255.255.255.255:9876` and `127.0.0.1:9876` every 3 seconds

#### Scenario: Beacon suppressed on UDP bind failure
- **WHEN** the UDP socket fails to bind
- **THEN** the relay SHALL continue operation without broadcasting
- **AND** the relay creation SHALL NOT be blocked

#### Scenario: Beacon contents match room metadata
- **WHEN** a beacon is broadcast
- **THEN** the `RoomAdvertisement` fields SHALL reflect the current room's metadata (room_name, map_id, current_players, max_players, state)
- **AND** the `relay_id` SHALL match the RelayHandle's relay_id

#### Scenario: Beacon received and room appears
- **WHEN** a UDP beacon is received by `LanDiscoveryListener`
- **THEN** the server SHALL appear in the LAN server list
- **AND** clicking it SHALL start the game connection

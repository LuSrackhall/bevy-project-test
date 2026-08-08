## MODIFIED Requirements

### Requirement: UDP beacon broadcast

The relay SHALL broadcast UDP beacons on the LAN at regular intervals. This applies to ALL relay implementations, including `ThreadRelayRuntime` (running as a background thread) and standalone `relay` binary.

The UDP socket SHALL bind to an **ephemeral port (`0.0.0.0:0`)** — NOT to the discovery listener port (9876) — and MUST have `set_broadcast(true)` enabled. Binding ephemeral avoids the `EADDRINUSE` conflict when the host's `LanDiscoveryListener` (which binds `0.0.0.0:9876` for browsing) is active in the room-list screen: a beacon bound to `0.0.0.0:9876` would fail to bind, silently disabling discovery. The beacon sends TO port 9876 on the broadcast targets; the receiving listener matches rooms by the packet's `relay_id`, not the source port.

Beacon errors MUST NOT prevent the relay from starting or operating. All UDP-related errors (bind, encode, send) SHALL be logged but silently ignored for relay operation.

#### Scenario: Beacon broadcast on relay start
- **WHEN** a relay starts (including `ThreadRelayRuntime`)
- **THEN** it SHALL broadcast a `LanDiscoveryPacket` to `255.255.255.255:9876`, the subnet broadcast `:9876`, and `127.0.0.1:9876` every 3 seconds, from an ephemeral source port

#### Scenario: Beacon does not conflict with the browsing listener
- **WHEN** the host is in the room-list screen (`LanDiscoveryListener` holds `0.0.0.0:9876`) and starts a relay
- **THEN** the beacon SHALL still bind successfully (ephemeral source port) and broadcast, so other machines on the LAN can discover the room

#### Scenario: Beacon suppressed on UDP bind failure
- **WHEN** the UDP socket fails to bind
- **THEN** the relay SHALL continue operation without broadcasting
- **AND** the relay creation SHALL NOT be blocked

#### Scenario: Beacon contents match room metadata
- **WHEN** a beacon is broadcast
- **THEN** the `RoomAdvertisement` fields SHALL reflect the current room's metadata (room_name, map_id, current_players, max_players, state)
- **AND** the `relay_id` SHALL match the RelayHandle's relay_id

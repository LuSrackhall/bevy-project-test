## MODIFIED Requirements

### Requirement: Relay responds with ReconnectResponse

The relay SHALL validate the `ruleset_version` match and respond with seed, map_spec_hash, and the command log from `last_tick_consumed + 1` to current tick, sent over the reliable Control channel. Logs exceeding the datagram MTU are transparently fragmented by the reliable layer (IPv6 ≤1232 payload) and reassembled on the client before replay. Application-level paging (`ReconnectPage`) is a future optimization for very long disconnects.

#### Scenario: schema mismatch

- **WHEN** the client's ruleset_version does not match the relay's
- **THEN** relay SHALL return INCOMPATIBLE status, client SHALL display version mismatch error

#### Scenario: successful reconnect response (full log)

- **WHEN** ruleset_version matches
- **THEN** relay SHALL respond with `ReconnectResponse` containing the full command log (`seed`, `map_spec_hash`, `first_tick`, `ticks`); logs over MTU are fragmented by the reliable layer and reassembled before delivery

#### Scenario: full response reassembled before replay

- **WHEN** the client has reassembled the full `ReconnectResponse`
- **THEN** the client SHALL apply the command log and resume replay from `first_tick`

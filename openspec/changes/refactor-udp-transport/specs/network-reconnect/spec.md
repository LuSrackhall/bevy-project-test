## MODIFIED Requirements

### Requirement: Relay responds with ReconnectResponse

The relay SHALL validate the `ruleset_version` match and respond with seed, map_spec_hash, and the command log from `last_tick_consumed + 1` to current tick, **paginated in chunks** (each page within MTU). The client SHALL fetch all pages before replaying.

#### Scenario: schema mismatch

- **WHEN** the client's ruleset_version does not match the relay's
- **THEN** relay SHALL return INCOMPATIBLE status, client SHALL display version mismatch error

#### Scenario: successful reconnect response (paginated)

- **WHEN** ruleset_version matches
- **THEN** relay SHALL respond with reconnect metadata (`seed`, `map_spec_hash`, `first_tick`, `page_count`), and the client SHALL fetch each page of `ticks` via `ReconnectPage` until all pages are received

#### Scenario: all pages fetched before replay

- **WHEN** the client has received all `ReconnectPage` responses
- **THEN** the client SHALL assemble the full command log and begin replay

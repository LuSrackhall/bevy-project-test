## MODIFIED Requirements

### Requirement: Relay responds with ReconnectResponse

The relay SHALL validate the `ruleset_version` match and respond to a `ReconnectRequest` with a metadata-only `ReconnectResponse` (`game_id`, `ruleset_version`, `seed`, `map_spec_hash`, `first_tick`, `total_ticks`, `page_count`, `players`), then push `page_count` `ReconnectPage` messages on the reliable Control channel. Pages SHALL cover contiguous tick ranges bucketed by tick value (`[first_tick + i*PAGE_TICKS, first_tick + (i+1)*PAGE_TICKS)`), NOT by log position (the finalized log is append-ordered, not tick-ordered). `total_ticks` SHALL count log entries with `tick > last_tick_consumed`. Logs exceeding a page's datagram MTU are fragmented by the reliable layer. The relay SHALL NOT wait for per-page requests (push model; ReliableOrdered preserves page order).

#### Scenario: schema mismatch

- **WHEN** the client's ruleset_version does not match the relay's
- **THEN** relay SHALL return INCOMPATIBLE status, client SHALL display version mismatch error

#### Scenario: successful reconnect response is metadata plus pages

- **WHEN** ruleset_version matches and the command log from `last_tick_consumed + 1` to current tick has `N` entries
- **THEN** relay SHALL respond with `ReconnectResponse { first_tick, total_ticks: N, page_count: ceil(N/PAGE_TICKS), seed, map_spec_hash, players }` and push `page_count` `ReconnectPage` messages on the Control channel

#### Scenario: page covers a contiguous tick range by value

- **WHEN** the log was finalized out of tick order (append order ≠ tick order) and a reconnect occurs
- **THEN** each `ReconnectPage` SHALL contain exactly the log entries whose ticks fall in its bucketed range, with no gaps or duplicates across pages, and `first_tick` of page `i+1` SHALL equal the last tick of page `i` plus one

#### Scenario: empty log returns no pages

- **WHEN** `last_tick_consumed` equals the current finalized tick (no new ticks)
- **THEN** relay SHALL respond with `page_count = 0` and no `ReconnectPage` messages

#### Scenario: reconnect during frozen game is rejected

- **WHEN** the game is frozen (timeout/GameOver path)
- **THEN** relay SHALL return an error rather than a command log

### Requirement: Client applies reconnect pages progressively

The client SHALL apply each `ReconnectPage` as it is received over the reliable Control channel, inserting its `ticks` into the relay buffer so the driver can resume replay before all pages arrive. The client SHALL track a page cursor and reject out-of-order, duplicate, or out-of-range pages, and SHALL verify `page.page_count` matches the metadata `page_count` (defense against stale pages from a previous session). When `page_count = 0`, the client SHALL consider replay complete immediately without waiting for pages.

#### Scenario: pages applied progressively in order

- **WHEN** the client receives the metadata `ReconnectResponse` then pages 0..n in order
- **THEN** each page's ticks are inserted as received; ticks in page 0 become ready for replay before page n arrives; the driver advances tick-by-tick as pages fill the buffer

#### Scenario: page validation rejects stale pages

- **WHEN** a `ReconnectPage` has a `page_index` not equal to the expected next page, a duplicate `page_index`, `page_index >= page_count`, or `page_count` different from the metadata's
- **THEN** the client SHALL reject the page (ignore it) and continue waiting for the expected page

#### Scenario: replay resumes from disconnect point

- **WHEN** the client reconnects after already applying some pages (`last_tick_consumed` advanced)
- **THEN** the reconnect request resumes from the applied tick; re-applied ticks SHALL NOT duplicate gaps or overlap in the relay buffer

#### Scenario: empty page set completes replay immediately

- **WHEN** the metadata reports `total_ticks = 0` / `page_count = 0`
- **THEN** the client SHALL mark replay complete without waiting for any `ReconnectPage`

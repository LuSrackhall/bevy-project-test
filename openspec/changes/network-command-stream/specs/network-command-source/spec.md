## ADDED Requirements

### Requirement: NetworkCommandSource implements CommandSource

The system SHALL provide a `NetworkCommandSource` struct that implements the `CommandSource` trait. It SHALL be the canonical command source for networked games.

- `is_tick_ready()` SHALL return true only when the relay's finalized `CommandBatch` for the given tick has been received and stored in `relay_buffer`
- `commands_for_tick()` SHALL return commands from `relay_buffer.remove(tick)`, NOT from the Bevy `cmd_buf` (ctx parameter SHALL be ignored in network mode)
- `should_record()` SHALL return true (network games always produce replay)
- The source SHALL NOT perform merge logic — it SHALL only consume relay-finalized batches
- Local player input SHALL still enter via `cmd_buf.push()` (uplink staging), but SHALL NOT be read back by `commands_for_tick()`

#### Scenario: commands_for_tick returns relay batch

- **WHEN** relay has broadcast `CommandBatch(tick=100)` and stored in `relay_buffer`
- **THEN** `commands_for_tick(100, ctx)` returns the commands from that batch, ignoring ctx

#### Scenario: is_tick_ready false before relay batch arrives

- **WHEN** tick 100 has not yet been finalized by relay
- **THEN** `is_tick_ready(100)` returns false

#### Scenario: is_tick_ready true after relay batch arrives

- **WHEN** relay buffer has finalized `CommandBatch(tick=100)`
- **THEN** `is_tick_ready(100)` returns true

#### Scenario: should_record always true in network mode

- **WHEN** `NetworkCommandSource` is active
- **THEN** `should_record()` returns true regardless of game state

#### Scenario: no merge with cmd_buf

- **WHEN** `commands_for_tick()` is called for any tick
- **THEN** the result SHALL NOT include commands from the Bevy `cmd_buf` resource

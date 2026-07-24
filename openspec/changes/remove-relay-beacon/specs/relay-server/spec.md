## ADDED Requirements

（无）

## MODIFIED Requirements

### Requirement: Relay server CLI

The relay CLI binary SHALL start a TCP relay server on a specified port.

(This replaces the previous requirement that included UDP beacon broadcast from the CLI.)

#### Scenario: Relay server starts
- **WHEN** `start_relay(port, seed, player_count)` is called
- **THEN** it SHALL bind a TCP listener on `0.0.0.0:{port}`
- **AND** delegate to `relay_core::run_relay()` for accept and client handling

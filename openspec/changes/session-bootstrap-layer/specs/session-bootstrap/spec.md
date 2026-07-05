## ADDED Requirements

### Requirement: SessionBootstrap pipeline

The system SHALL provide a standardized initialization pipeline: `GameIntent → resolve_intent() → SessionConfig → dispatch() → SessionArtifacts → wire() → Driver`.

- `GameIntent` SHALL be defined in render_view (UI semantic type)
- `resolve_intent()` SHALL be a pure function in render_view, returning `SessionConfig`
- `dispatch()` SHALL route SessionConfig to the appropriate initializer and return `SessionArtifacts` (enum)
- `wire()` SHALL consume `SessionArtifacts` by move and register all resources
- `wire()` SHALL NOT call any initializer

#### Scenario: Single mode pipeline

- **WHEN** `GameIntent::Single { map_size }` is created
- **THEN** pipeline produces `SessionArtifacts::Live` and `wire()` sets `CommandSource::Live`

#### Scenario: Replay mode pipeline

- **WHEN** `GameIntent::Replay { path }` is created
- **THEN** pipeline produces `SessionArtifacts::Replay { replay }` and `wire()` sets `CommandSource::Replay`

#### Scenario: Network mode pipeline

- **WHEN** `GameIntent::Network { relay_addr, player_count }` is created
- **THEN** pipeline connects to relay, produces `SessionArtifacts::Network(result)`, wire sets `CommandSource::Network`

### Requirement: Bootstrap is one-shot and non-reentrant

Bootstrap SHALL be guarded by `BootstrapPhase`: entry checks `phase == Init`, exit sets `phase = Wired`. Re-entrant calls SHALL be silently skipped.

#### Scenario: bootstrap re-entrance

- **WHEN** `bootstrap()` is called while `phase != Init`
- **THEN** it SHALL return immediately without side effects

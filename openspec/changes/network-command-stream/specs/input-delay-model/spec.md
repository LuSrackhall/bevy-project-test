## ADDED Requirements

### Requirement: Input delay defaults to 3 ticks

The system SHALL apply an input delay of `D` ticks before player commands are executed in network mode. The default value of `D` SHALL be 3 ticks.

At 20Hz tick rate (50ms per tick), 3 ticks = 150ms.

#### Scenario: default input delay applied

- **WHEN** a player issues a command via `cmd_buf.push()` at tick 100
- **THEN** the command SHALL be assembled into `PlayerTickFrame` targeting tick 103 (= 100 + D)

### Requirement: Input delay is configurable

The input delay `D` SHALL be configurable via a parameter, allowing adjustment based on network conditions.

#### Scenario: input delay overridden

- **WHEN** the input delay is configured to 5 ticks
- **THEN** commands pushed at tick 100 SHALL target tick 105

### Requirement: Input delay formula

The input delay SHALL satisfy: `D >= R / T_tick + J`, where:
- `R` = RTT 95th percentile (ms)
- `T_tick` = tick duration (50ms @ 20Hz)
- `J` = jitter buffer (1 tick minimum)

Default D=3 covers `R <= 100ms`.

#### Scenario: formula rejects insufficient delay

- **WHEN** configured `D = 2` but `R / T_tick + J = 3.5`
- **THEN** the system SHALL clamp D to at least 4 (ceiling of 3.5)

### Requirement: Delay offset occurs only inside NetworkCommandSource

The tick offset SHALL be applied exclusively inside `NetworkCommandSource`. `render_view` code SHALL continue to push commands with `tick = current + 1` (no changes to existing input code).

#### Scenario: render_view tick unaffected

- **WHEN** `render_view` calls `cmd_buf.push({ tick: clock.current_tick + 1, ... })` during network mode
- **THEN** `NetworkCommandSource` SHALL internally map this to the delayed tick before sending to relay

### Requirement: Timeout uses relay wall clock

The tick timeout SHALL be based on relay wall clock, measured from `first_arrival[tick]` (the moment the first `PlayerTickFrame` for that tick reaches the relay). Timeout value = `D * T_tick * 1000 + jitter_ms`.

#### Scenario: timeout triggered

- **WHEN** `now_ms() - first_arrival[100] >= D * T_tick * 1000 + jitter_ms`
- **THEN** tick 100 SHALL be finalized via timeout, with NoOp for missing players

---

**Implementation:** `NetworkCommandSource.input_delay` field + `delayed_tick()` method (network.rs). RelayServer uses `input_delay` for timeout calculation. Configurable via constructor parameter.

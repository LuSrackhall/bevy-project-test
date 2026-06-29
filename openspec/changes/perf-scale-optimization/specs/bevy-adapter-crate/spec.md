## ADDED Requirements

### Requirement: tracing instrumentation around tick

bevy_adapter SHALL add tracing spans around the simulation tick call. Tracing dependencies (tracing, tracing-tracy) SHALL be gated behind a Cargo feature.

#### Scenario: tracing enabled
- **WHEN** bevy_adapter is compiled with the tracing feature and Tracy client is connected
- **THEN** each tick SHALL produce a trace span visible in Tracy

#### Scenario: tracing disabled
- **WHEN** bevy_adapter is compiled without the tracing feature
- **THEN** zero tracing code SHALL be included in the binary

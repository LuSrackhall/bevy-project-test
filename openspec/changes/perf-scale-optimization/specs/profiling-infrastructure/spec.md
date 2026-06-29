## ADDED Requirements

### Requirement: bevy_adapter tracing instrumentation

bevy_adapter SHALL add tracing spans around the `run_tick_default()` call in the simulation driver. Tracing SHALL be gated behind a feature flag. Simulation crate SHALL have zero tracing dependencies.

#### Scenario: tracing span around tick
- **WHEN** the tracing feature is enabled and run_tick_default() is called
- **THEN** a `tracing::info_span!("tick", tick_number)` SHALL surround the call

#### Scenario: tracing disabled in release
- **WHEN** the tracing feature is disabled
- **THEN** no tracing code SHALL be compiled into bevy_adapter

#### Scenario: simulation purity
- **WHEN** simulation crate is compiled
- **THEN** it SHALL NOT depend on tracing, tracy, Instant, or SystemTime

### Requirement: tracy subscriber registration

bevy_adapter SHALL register a `tracing-tracy` subscriber when the tracing feature is enabled. The subscriber SHALL forward tracing spans to Tracy for visualization.

#### Scenario: tracy connection
- **WHEN** the application runs with tracing feature enabled and Tracy client connected
- **THEN** tick spans SHALL appear in Tracy's timeline

### Requirement: debug_render feature gate

render_view SHALL gate `draw_debug_shapes_system` and `unit_info_bar_system` behind a `debug_render` Cargo feature. Feature name SHALL match constitution §21.

#### Scenario: debug_render enabled
- **WHEN** render_view is compiled with debug_render feature
- **THEN** debug shapes and unit info bars SHALL be registered and rendered

#### Scenario: debug_render disabled
- **WHEN** render_view is compiled without debug_render feature
- **THEN** debug shapes and unit info bars SHALL NOT be compiled or registered

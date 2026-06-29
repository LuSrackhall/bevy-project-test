## ADDED Requirements

### Requirement: debug_render feature gate

render_view SHALL gate debug visualization systems behind a `debug_render` Cargo feature. This includes draw_debug_shapes_system and unit_info_bar_system.

#### Scenario: feature disabled
- **WHEN** render_view is compiled without debug_render
- **THEN** debug shapes and unit info bars SHALL NOT be registered or rendered

#### Scenario: feature enabled
- **WHEN** render_view is compiled with debug_render
- **THEN** all debug visualization systems SHALL function as before

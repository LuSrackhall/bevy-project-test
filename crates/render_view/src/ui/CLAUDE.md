# Bevy UI Architecture Guidelines

Follow the official Bevy UI architecture and favor architectural principles over temporary APIs.

## Architecture

- Build UI from composable behavioral primitives (widgets or equivalent abstractions).
- Treat widgets as behavioral building blocks, not business objects.
- Keep business logic outside widgets.
- Keep business state in ECS Resources or domain Components.
- Treat UI state as transient presentation state, never as application state.
- Widgets are a projection of application state and do not own it.
- Keep behavior, presentation, and business logic decoupled.

## Interaction

- Prefer the official event-driven interaction model.
- React to semantic UI events or declarative state updates instead of polling visual state.
- Never drive gameplay or application logic from transient UI state.
- Prefer the official input pipeline over custom input handling.
- Design interaction to support pointer input, focus navigation, keyboard, gamepad, touch, and accessibility.

## Widget Design

Widgets are responsible only for:

- interaction behavior
- focus and navigation
- accessibility
- transient UI state
- semantic event emission

Widgets must not own domain or gameplay state, and must never contain:

- gameplay logic
- business rules
- application state
- project-specific behavior

## Future Compatibility

Do not couple application architecture to Bevy's current implementation details.

When Bevy evolves (input model, UI architecture, declarative UI system, ECS relationships, or runtime UI model), preserve these architectural principles instead of preserving specific APIs or implementation patterns.

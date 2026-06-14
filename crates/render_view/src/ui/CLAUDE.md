# Bevy UI Architecture Guidelines

Follow the official Bevy UI architecture and favor architectural principles over temporary APIs.

## Architecture

* Use official widgets as the behavioral foundation.
* Build UI from reusable, composable widgets.
* Treat widgets as behavioral building blocks, not business objects.
* Keep business logic outside widgets.
* Keep business state in ECS Resources or domain Components.
* Treat UI state as transient presentation state, never as application state.
* Widgets reflect application state; they never own the source of truth.
* Keep behavior, presentation, and business logic decoupled.

## Interaction

* Prefer the official event-driven interaction model.
* React to semantic UI events instead of polling visual state.
* Never drive gameplay or application logic from transient UI state.
* Prefer the official input pipeline over custom input handling.
* Design interaction to support mouse, keyboard, gamepad, touch, and accessibility.

## Widget Design

Widgets are responsible only for:

* interaction behavior
* focus and navigation
* accessibility
* transient UI state
* semantic event emission

Widgets must never contain:

* gameplay logic
* business rules
* application state
* project-specific behavior

## Future Compatibility

Do not couple application architecture to Bevy's current implementation details.

When Bevy evolves (Observers, BSN, Relations, UI runtime, or future UI systems), preserve these architectural principles instead of preserving specific APIs or implementation patterns.

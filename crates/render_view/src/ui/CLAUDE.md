# Bevy UI Guidelines

Follow the official Bevy UI architecture instead of legacy UI patterns.

## Architecture

- Use official widgets as the behavioral foundation.
- Treat widgets as headless behavior, not visual components.
- Keep business logic outside widgets.
- Keep business state in ECS Resources or domain Components.
- Treat widget state as transient UI state, never as business state.
- Let widgets reflect application state, never own it.
- Style belongs to the application layer.

## Interaction

- Prefer the official event-driven interaction model.
- React to semantic widget events instead of polling visual state.
- Never build gameplay logic around transient widget state.
- Prefer the official pointer interaction pipeline over custom input handling.
- Design UI to support mouse, keyboard, gamepad, touch, and accessibility.

## Widget Design

Widgets are responsible only for:

- interaction behavior
- focus
- accessibility
- keyboard navigation
- transient UI state
- semantic event emission

Widgets must never contain:

- gameplay logic
- business rules
- application state
- project-specific styling

## Compatibility

Prefer architectural principles over specific APIs.

When Bevy evolves (Observers, BSN, Relations, or future UI systems), preserve these architectural principles instead of preserving old implementation details.

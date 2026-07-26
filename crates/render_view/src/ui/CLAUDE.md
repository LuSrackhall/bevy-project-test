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

## Bevy 0.19 UI 开发参考

Bevy UI 功能开发时（特别是文本输入、焦点管理、Widget），优先查阅官方文档而非 AI 训练数据：

- **Bevy API 文档（主入口）**：<https://docs.rs/bevy/0.19.0/bevy/index.html>
- **UI Widgets**：`bevy::ui_widgets` 模块（Button, EditableText 等）
- **文本输入（EditableText）**：<https://docs.rs/bevy/0.19.0/bevy/text/struct.EditableText.html>
  - 示例：`<bevy_repo>/examples/ui/text/text_input.rs`
- **输入焦点管理**：`bevy::input_focus` 模块（AutoFocus, TabGroup, TabIndex, InputFocus）
- **光标样式**：`bevy::text::TextCursorStyle`

## Context

当前创建房间弹窗中，房间名输入是一个硬编码"默认房间名"按钮，用户无法自定义房间名（Issue #10）。Bevy 0.19 提供了原生 `EditableText` widget（`bevy::text::EditableText`），项目已启用 `bevy_ui_widgets` 和 `bevy_input_focus` features。

## Goals / Non-Goals

**Goals：**
- 将 `ModalRoomName` 按钮替换为 `EditableText` 输入框
- 弹窗打开时自动聚焦输入框（`AutoFocus`）
- 创建房间时读取 `EditableText::value()` 作为房间名
- 更新 UI CLAUDE.md 添加 Bevy 0.19 文档参考

**Non-Goals：**
- 地图选择 / 人数选择 UI 改进（保持当前按钮循环）

## Decisions

### D1: 使用 EditableText 替代按钮

```rust
// 旧：WidgetButton + Text("默认房间名")
// 新：EditableText { visible_width: Some(15), allow_newlines: false } + AutoFocus
```

标签"房间名:"保留。输入框使用 `ModalRoomName` 标记。

### D2: 创建时读取 EditableText

```rust
// 旧：request.room_name = state.room_name.clone();
// 新：从 Query<&EditableText, With<ModalRoomName>> 获取 value()
```

### D3: ModalState.room_name 保留

`ModalState` 结构体保留不改，避免改动其他引用。但 room_name 的值不再由 UI 维护。

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| EditableText 需要 AutoFocus/InputFocus 插件支持 | 项目已有 bevy_input_focus feature，添加 TabNavigationPlugin |
| 键盘输入在 WASM 环境中需额外处理 | 不影响当前 MVP（桌面优先） |

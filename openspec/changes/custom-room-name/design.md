## Context

设计细节见 [brainstorm-spec.md](brainstorm-spec.md)。变更极小——单文件修改 `lan_lobby.rs`。

## Decisions

### D1: EditableText 配置

```rust
EditableText {
    visible_width: Some(15.),
    allow_newlines: false,
    ..default()
}
```

宽度 15 字符（约 200px UI 单位），单行输入。添加 `AutoFocus` 使弹窗打开时输入框自动获得焦点。

### D2: 创建房间时读取输入

在 `open_create_room_modal` 的创建按钮 observer 中，新增 `Query<&EditableText, With<ModalRoomName>>` 获取当前输入值：

```rust
if let Ok(editable) = editable_text_q.get_single() {
    request.room_name = editable.value().to_string();
}
```

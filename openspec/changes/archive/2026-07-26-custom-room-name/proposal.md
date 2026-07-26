## Why

创建房间弹窗中房间名输入是硬编码按钮，用户无法自定义房间名。当前 `handle_create_room` 中已有自动生成默认名的回退逻辑（`format!("房间_{}", timestamp)`），但用户应能输入自己的房间名。

## What Changes

- `ModalRoomName` 按钮 → `EditableText` 输入框 + `AutoFocus`
- 创建按钮 observer 改为读取 `EditableText::value()` 而非 `ModalState.room_name`
- `render_view/Cargo.toml` 已包含所需 features，无需改动

## Capabilities

### Modified Capabilities
- `lan-room-list`: 房间名输入从按钮改为 EditableText

## Impact

| 范围 | 文件 | 说明 |
|---|---|---|
| 修改 | `render_view/src/ui/lan_lobby.rs` | ModalRoomName 按钮 → EditableText + AutoFocus |
| 修改 | `render_view/src/ui/CLAUDE.md` | 添加 Bevy 0.19 UI 开发参考 |

## Why

LAN 房间列表的加入按钮点击无响应。根因是 `update_room_list` 每帧全量 despawn + respawn 所有行，导致 Bevy 0.19 的 `Pressed` 组件在 `Pointer<Press>`→`Pointer<Click>` 之间丢失，`Activate` 永不触发。项目中所有其他按钮均为一次性 spawn，实体跨帧稳定，故正常工作。

## What Changes

将 `update_room_list` 从全量重建改为增量更新：
- 新增 `LanLobbyRowData(RelayId)` 组件用于行身份匹配
- 新增 4 个 Text 标记组件（`RoomNameLabel` 等）用于文本更新定位
- 增量更新逻辑：移除消失行 → 添加新行 → 更新存量行文本
- 对 `servers.servers` 按 `relay_id` 排序保证顺序稳定
- 还原 observer 为 `On<Activate>` 在 WidgetButton 实体上

## Capabilities

### New Capabilities
- `incremental-room-list`: 增量更新 LAN 房间列表，保持按钮实体跨帧稳定

### Modified Capabilities

无。不修改现有 spec 需求。

## Impact

- 仅修改 `crates/render_view/src/ui/lan_lobby.rs`
- 不影响 `bevy_adapter` 或 `simulation` 层
- 不影响 LAN 发现协议或网络传输
- 不影响项目其他 UI 按钮（保持 `WidgetButton` + `On<Activate>` 统一模式）

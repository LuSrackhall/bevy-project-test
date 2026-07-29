## Why

加入者获得的 player_id 始终为 0（与主机相同），导致双方操控同一阵营。根因是 `GameJoined` 事件未同步更新 `NetworkGameStart.player_id`，`reset_game_system` 使用了 `setup_lobby_system` 中设置的临时值 `0`。

## What Changes

- `lobby_update_system` 的 `GameJoined` handler 新增 `network_start.*` 赋值
- `JoinRoomRequest` 新增 `max_players: u8` 字段
- `handle_join_room` 用 `request.max_players` 替代硬编码 `2u8`
- `lan_lobby.rs` observer 读取 `room.max_players` 写入 `JoinRoomRequest`

## Capabilities

### New Capabilities
- `player-identity-join`: 加入者 player_id 正确分配流程

### Modified Capabilities
无。不修改现有 spec 需求。

## Impact

- `crates/render_view/src/lib.rs`：4 处改动
- `crates/render_view/src/ui/lan_lobby.rs`：observer 加 1 行

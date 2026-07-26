## Why

C1 修复了 Lobby 协议断裂（JoinGame 发送、房主进入 Lobby、LobbyUpdate 处理），但房间等待页 UI 仍然是基础状态：仅有标题和状态文本，没有玩家列表可视化、没有就绪反馈、没有房主"开始游戏"按钮。C2 补齐这些 UI 缺口。

## What Changes

- 玩家列表动态渲染：`update_lobby_player_list` 系统，复用 lan_lobby.rs 的 update_room_list 模式
- 就绪按钮点击后视觉反馈（文本改为"已就绪"）
- 房主显示"开始游戏"按钮（非房主显示"就绪"按钮）
- `setup_lobby_ui` 增加玩家列表容器节点

## Capabilities

### Modified Capabilities
- `lobby-ui-v2`: 房间等待页玩家列表 + 就绪/开始按钮

## Impact

| 范围 | 文件 | 说明 |
|---|---|---|
| 修改 | `render_view/src/ui/lobby.rs` | 新增玩家列表容器、update_lobby_player_list、就绪/开始按钮逻辑 |
| 修改 | `render_view/src/ui/mod.rs` | 注册 update_lobby_player_list 系统 |

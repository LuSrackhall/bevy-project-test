## Why

Lobby 协议层（LobbyReady/LobbyUpdate/GameStarted）已完成，但 UI 层仍停留在 V1（只有连接状态文本和取消按钮）。需补全 Ready 按钮、玩家就绪列表、LobbyUpdate 轮询以支持大厅完整交互。

## What Changes

1. NetworkEvent 新增 LobbyUpdate 变体（复用接收通道）
2. NetworkSender 新增 send_lobby_ready() 方法
3. run_session() 中 LobbyUpdate 推送到 event_receiver
4. LobbyPhase 新增 Ready 阶段
5. UI 层添加 Ready 按钮 + 玩家就绪列表

## Capabilities

### New Capabilities

- `lobby-ui-v2`: 联机大厅 V2 UI（Ready 按钮 + 玩家列表 + 事件轮询）

## Impact

- crates/bevy_adapter/src/network.rs — NetworkEvent 变体 + NetworkSender 扩展
- crates/bevy_adapter/src/transport.rs — LobbyUpdate → push + write task 检查 lobby_ready
- crates/render_view/src/lib.rs — LobbyPhase::Ready + LobbyUpdate 轮询
- crates/render_view/src/ui/lobby.rs — Ready 按钮 + 玩家列表 UI

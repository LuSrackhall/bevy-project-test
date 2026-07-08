## Why

当前 relay 协议缺少 Lobby 阶段交互。玩家连接后立即开始游戏，无法在 Lobby 中编辑槽位、选择地图或 Ready 握手。需扩展协议以支持大厅 V2。

## What Changes

RelayClientMessage 新增 LobbyReady, RelayServerMessage 新增 LobbyUpdate。Relay 端增加 lobby ready 追踪逻辑。

## Capabilities

### New Capabilities

- `relay-lobby-protocol`: Relay 协议层支持 Lobby 阶段消息（Ready/LobbyUpdate）

## Impact

- `crates/bevy_adapter/src/network.rs` — 2 个枚举变体 + LobbyPlayerState 结构体
- `crates/relay/src/lib.rs` — lobby ready 追踪 + 新 match arm
- `crates/bevy_adapter/src/transport.rs` — LobbyUpdate match arm (忽略)
- `crates/relay/tests/two_client_sync.rs` — 双玩家 lobby 测试

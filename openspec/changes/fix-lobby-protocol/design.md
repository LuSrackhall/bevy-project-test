## Context

高维设计见 [brainstorm-spec.md](brainstorm-spec.md)。C1 是#11 的底层修复阶段，修复三个阻塞级协议断裂后再构建完整等待页 UI（C2）。

## Goals / Non-Goals

**Goals：**
- `NetworkSender` 增加 JoinGame 发送（TCP 连接后、run_session 前）
- `NeedsGameReset::Network` 增加 `relay_id: RelayId`
- `handle_create_room` 成功 → 设 NeedsGameReset + GameState::Lobby
- 引入 `IsHost(bool)` Resource
- 修复 `lobby_update_system` 的 LobbyUpdate 处理

**Non-Goals：**
- 玩家列表 UI 渲染（C2）
- 房主"开始游戏"按钮（C2）
- JoinRejected 事件映射（C3）
- 取消就绪（C3）

## Decisions

### D1: JoinGame 发送位置

在 `spawn_network_client` 的 tokio 线程中，TCP 连接成功、进入 `run_session` 之前。

```rust
// 新增: 发送 JoinGame
let join_msg = RelayClientMessage::JoinGame { room_id: RoomId(0), relay_id };
let data = bincode::serde::encode_to_vec(&join_msg, bincode::config::standard())?;
let len_bytes = (data.len() as u32).to_le_bytes();
stream.write_all(&len_bytes).await?;
stream.write_all(&data).await?;

// 原有的 run_session 循环
run_session(reader, writer, receiver, sender, event_receiver).await;
```

`spawn_network_client` 和 `spawn_network_client_nonblocking` 都需要增加 `relay_id: RelayId` 参数。

### D2: handle_create_room 修改

```rust
fn handle_create_room(
    mut request: ResMut<CreateRoomRequest>,
    mut controller: ResMut<SessionController>,
    mut needs_reset: ResMut<NeedsGameReset>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if !request.requested { return; }
    request.requested = false;

    // 构建 RoomMetadata...
    match controller.create_session(room) {
        Ok(_) => {
            let session = controller.current_session().unwrap();
            let endpoint = session.relay.endpoint();
            let relay_id = session.relay.relay_id();
            *needs_reset = NeedsGameReset::Network {
                relay_addr: format!("127.0.0.1:{}", endpoint.port()),
                player_count: request.max_players,
                player_id: Some(0),
                relay_id,
            };
            commands.insert_resource(IsHost(true));
            next_state.set(GameState::Lobby);
        }
        Err(e) => bevy::log::error!("[LAN] Failed to create room: {}", e),
    }
}
```

### D3: NeedsGameReset 扩展

```rust
pub enum NeedsGameReset {
    None,
    SameSize,
    NewGame(MapSize),
    Replay(ReplayFile),
    Network {
        relay_addr: String,
        player_count: u8,
        player_id: Option<u8>,
        relay_id: RelayId,  // 新增
    },
}
```

### D4: IsHost 资源 + LobbyPlayerList

```rust
#[derive(Resource)]
pub struct IsHost(pub bool);

#[derive(Resource, Default)]
pub struct LobbyPlayerList(pub Vec<bevy_adapter::network::LobbyPlayerState>);
```

### D5: LobbyUpdate 修复

`lobby_update_system` 收到 LobbyUpdate 后：
1. 存储 `players` 到 `LobbyPlayerList`
2. 查找本地玩家 (`player_id == local_player_id`)
3. 仅当本地 ready == true 才设 `LobbyPhase::Ready`

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| JoinGame 发送失败导致 TCP 连接断开 | tokio 线程退出，LobbyPhase 保持在 Connecting，用户可取消重试 |
| relay_id 传递链长（handle_join → NeedsGameReset → spawn_client） | 数据流清晰，单向传递无循环 |
| 环回 TCP 增加房主延迟 | 毫秒级，可忽略 |

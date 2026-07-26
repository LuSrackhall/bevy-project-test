## Context

C1 修复了 Lobby 协议的三个阻塞级断裂（JoinGame 发送、房主进入 Lobby、LobbyUpdate 处理）。C2 在此基础上构建完整的房间等待页 UI。

### 当前状态

- `lobby.rs`: 仅有标题 + 状态文本 + 取消/就绪按钮
- `LobbyPlayerList(Vec<LobbyPlayerState>)`: 资源已就绪（C1 引入），由 lobby_update_system 在收到 LobbyUpdate 时填充
- `IsHost(bool)`: 资源已就绪（C1 引入），用于房主判定

### 缺失

- 玩家列表可视化（看不到谁进了房间、谁就绪了）
- 就绪按钮的视觉反馈（点了就绪不知道是否成功）
- 房主"开始游戏"按钮（当前只有"就绪"按钮，对房主不够直观）
- 动态玩家列表与 LobbyPlayerList 资源的同步

## Goals / Non-Goals

**Goals：**
- 玩家列表动态渲染（player_id + 就绪状态图标）
- 就绪按钮点击后视觉反馈
- 房主"开始游戏"按钮（发送 LobbyReady 触发全部就绪）
- 系统注册 + 生命周期管理

**Non-Goals：**
- 取消就绪 toggle（需 relay_core 协议扩展，属 C3）
- 房主强制开始协议（与就绪按钮合并）
- 自定义房间名显示
- 地图选择

## Decisions

### D1: 玩家列表渲染模式

复用 `lan_lobby.rs` 的 `update_room_list` 模式：

```
LobbyPlayerListContainer (Entity, marker) 
  ├── LobbyPlayerRow (player_id=0, ready=✅)
  ├── LobbyPlayerRow (player_id=1, ready=❌)
  └── ...
```

`update_lobby_player_list` 系统每帧同步 `LobbyPlayerList` 资源 → despawn 旧行 → spawn 新行。

### D2: 就绪按钮行为

点击就绪 → 发送 `LobbyReady(true)` → 按钮标记为已就绪（文本变灰或显示"已就绪"）。不可取消（C3 实现取消）。

### D3: 房主开始游戏按钮

房主看到"开始游戏"按钮而非"就绪"。点击后发送 `LobbyReady(true)`（房主 player_id=0）。当所有玩家（包括房主）就绪后，relay 广播 GameStarted。

### D4: 系统注册

```rust
// mod.rs
.add_systems(Update, lobby::update_lobby_player_list
    .after(crate::lobby_update_system)
    .run_if(in_state(crate::GameState::Lobby)))
```

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| 玩家列表在 Connecting 阶段为空 | `update_lobby_player_list` 在列表为空时 despawn 所有行，不报错 |
| 房主按钮与"开始游戏"语义差距 | 点击"开始游戏" = 发送 LobbyReady，本质是隐式就绪；后续 C3 可加 ForceStart 协议 |

## Context

高维设计见 [brainstorm-spec.md](brainstorm-spec.md)。C2 在 C1 的 LobbyPlayerList + IsHost 基础上构建完整 UI。

## Goals / Non-Goals

**Goals：**
- 玩家列表容器 + `update_lobby_player_list` 系统
- 就绪按钮视觉反馈
- 房主"开始游戏"按钮
- 系统注册

**Non-Goals：**
- 取消就绪 toggle（C3）
- 房主强制开始协议

## Decisions

### D1: 玩家列表容器

在 `setup_lobby_ui` 的状态文本和按钮之间插入容器节点：

```
Node { flex_grow: 1.0, flex_direction: Column }
  → LobbyPlayerListContainer (marker component)
```

`update_lobby_player_list` 系统使用 `LobbyPlayerListContainer` 标记找到容器，清空旧行，spawn 新行。

### D2: update_lobby_player_list 实现

复用 `lan_lobby.rs` 的 `update_room_list` 模式：
1. Query `LobbyPlayerListContainer` 获取容器 Entity
2. Despawn 所有 `LobbyPlayerRow` 标记的行
3. 遍历 `LobbyPlayerList` 资源中的 players
4. 每行显示：`Player {id}` + 就绪状态（✔ / ✘）

### D3: 就绪/开始按钮

- 非房主：显示"就绪"按钮 → 点击发送 LobbyReady(true) → 按钮文本改为"已就绪"
- 房主：显示"开始游戏"按钮 → 点击发送 LobbyReady(true) → 按钮文本改为"已开始"
- 按钮可见性通过 `toggle_button_visibility` 系统按 `IsHost` 切换（ Display::Flex / Display::None ）
- `ReadyState(bool)` Resource 跟踪本地就绪状态，`OnEnter(Lobby)` 时重置为 false

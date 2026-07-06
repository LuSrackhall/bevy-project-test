## Context

当前 Lobby 状态（GameState::Lobby）在 `render_view/src/lib.rs` 中已注册：

- `OnEnter(Lobby)` → `setup_lobby_system`：TCP 连接 + `bootstrap_session`（**阻塞主线程**）
- `Update(Lobby)` → `lobby_wait_system`：轮询 GameStarted

问题：`setup_lobby_system` 调用 `spawn_network_client` 时在 `transport.rs:236` 的 `connected_rx.recv_timeout(30s)` 阻塞，冻结渲染。

## Goals / Non-Goals

**Goals：**
- TCP 连接异步轮询，不阻塞主线程
- Lobby UI 显示连接状态（Connecting / Connected / Failed）
- 主菜单联机按钮正确指向 Lobby
- 取消按钮返回主菜单
- 保持单机快速开始 / 回放功能不变

**Non-Goals：**
- 槽位编辑 / 阵营选择（V2）
- Ready / Start 握手（V2）
- Relay 协议扩展

## Decisions

### D1：非阻塞 TCP 连接

方案：将 `spawn_network_client` 拆分为发起连接和轮询结果两步。

```rust
pub struct LobbyConnectionStatus {
    pub result: Arc<Mutex<Option<Result<(), String>>>>,
}
```

`setup_lobby_system` 调用 `spawn_network_client_async`，该函数启动 tokio 线程后立即返回，不阻塞。连接状态存入 `LobbyConnectionStatus` Resource。

每帧 Update 系统轮询 `LobbyConnectionStatus.result`，根据状态驱动 UI：

| 状态 | UI |
|------|-----|
| `None`（连接中） | 显示 "正在连接..." + 旋转/进度 |
| `Some(Ok(()))` | 连接成功 → 显示 "等待其他玩家..." + 转入 GameStarted 等待 |
| `Some(Err(e))` | 连接失败 → 显示错误信息 + "返回"按钮 |

### D2：Lobby UI 结构

新文件 `render_view/src/ui/lobby.rs`，单一 `LobbyUI` marker component。

布局：
```
+----------------------------------+
|  联机大厅           [返回]       |
|                                   |
|  ● 正在连接中...                 |
|  [取消]                          |
|                                   |
|  或                              |
|                                   |
|  ✓ 已连接，等待其他玩家...       |
|  当前玩家: 1/2                    |
|  [取消]                          |
|                                   |
|  或                              |
|                                   |
|  ✗ 连接失败                      |
|  无法连接到 127.0.0.1:9876       |
|  [返回主菜单]                    |
+----------------------------------+
```

### D3：主菜单联机按钮

`menu.rs` 中"开始联机"按钮的 `observer` 改为：

```rust
*needs_reset = NeedsGameReset::Network { relay_addr, player_count, player_id };
next.set(GameState::Lobby);  // 原来是 Playing
```

### D4：状态转换

```
MainMenu → Lobby → (OnEnter) TCP连接发起
                → (Update) Connecting → Connected
                → (Update) Connected + GameStarted → Playing
                → (Update) Failed → 错误显示 → [返回] → MainMenu
```

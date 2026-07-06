## Context

当前 `GameState::Lobby` 已存在但子菜单的"开始联机"按钮跳过它直达 `Playing`（`menu.rs:152`）。Lobby 状态仅有 TCP 连接和 GameStarted 等待逻辑，无任何 UI渲染（白屏）。TCP 连接阻塞主线程（`transport.rs:236`，30s 超时），导致即使进入 Lobby 也无法渲染任何界面。

联机大厅 UI 是连接「主菜单」和「锁步联机游戏」之间的视觉桥梁。

## Goals / Non-Goals

### Goals（本变更负责）

- 主菜单"联机"按钮正确指向 `GameState::Lobby`
- Lobby 有 UI：连接状态显示 + "等待其他玩家..." + 取消按钮返回菜单
- TCP 连接改为异步轮询，不阻塞 Bevy 主线程
- 保持所有现有功能不变（单机快速开始、回放）
- 测试覆盖：Lobby 状态转换 + TCP 异步连接 + UI 显示

### Non-Goals

- 槽位编辑 / 阵营选择（需要 relay 协议扩展——V2）
- 地图选择（需要 relay 协议扩展——V2）
- Ready / Start 握手（V2）
- 局域网发现
- Relay 协议扩展

## Decisions

### D1：TCP 连接异步化

**问题：** `spawn_network_client` 阻塞等待 TCP 连接（`connected_rx.recv_timeout(30s)` 在 `transport.rs:236`），冻结 Bevy 渲染。

**方案：** 将 `connected_rx` 作为 Resource 返回，不阻塞。

```rust
pub fn spawn_network_client_nonblocking(...) -> Result<
    (NetworkReceiver, NetworkSender, NetworkClientHandle, ConnectionStatusRx), String
>
```

`ConnectionStatusRx` 是一个 Bevy Resource，包装 `Arc<Mutex<Option<Result<(), String>>>>`。每帧轮询。

原有 `spawn_network_client` 保留（用于现有测试），新增非阻塞变体。

### D2：Lobby UI 状态机

```rust
#[derive(Resource)]
pub struct LobbyState {
    pub phase: LobbyPhase,
    pub map_size: MapSize,
}

pub enum LobbyPhase {
    Connecting,    // TCP 连接中 → 显示 "正在连接..."
    Connected,     // TCP 已连接，等待 GameStarted → 显示 "等待其他玩家..."
    Failed(String),// 连接失败 → 显示错误 + "返回"按钮
}
```

`LobbyState` 是本地 Resource（render_view 层），不写入 simulation。

### D3：主菜单 -> Lobby -> Playing 三阶段

```
主菜单 "联机"
  → next.set(Lobby)
  → OnEnter(Lobby): 发起 TCP 连接(非阻塞)
  → Update(Lobby):
      Phase=Connecting → 轮询连接状态 → 成功则 Phase=Connected
      Phase=Connected → 系统轮询 GameStarted → 收到则 next.set(Playing)
      Phase=Failed → 显示错误
  → OnEnter(Playing): reset_game_system 用网络种子创建世界
  → 正常游戏
```

### D4：菜单按钮修复

`menu.rs:152` 从 `next.set(Playing)` 改为 `next.set(Lobby)`。移除旧的笨拙文本输入（relay地址、player count、player ID），改为简洁的单个"联机"按钮。

### D5：测试策略（TDD）

1. **单元测试**：`LobbyState` 状态转换逻辑
2. **集成测试**：Lobby → Playing 状态转换（模拟 GameStarted 事件）
3. **非阻塞连接测试**：模拟 TCP 连接成功/失败路径

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| TCP 连接异步化可能引入竞态 | `Arc<Mutex<Option<>>>` 跨线程安全，每帧轮询一次 |
| Lobby UI 过于简单（只有"等待中"） | 这是有意的 MVP 裁剪——V2 通过 relay 协议扩展增加完整功能 |
| 连接失败时的恢复路径 | `LobbyPhase::Failed`→显示错误→"返回"按钮→MainMenu |
| 与现有 bootstrap 管线的兼容性 | 不修改 bootstrap 流程，仅将同步 TCP 改为异步轮询 |

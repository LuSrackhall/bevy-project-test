## Context

当前 Lobby UI（V1）只有连接状态文本和取消按钮。协议层（`relay-lobby-protocol`）已支持 `LobbyReady`/`LobbyUpdate`/`GameStarted`，但客户端 UI 未使用这些能力。三个子 Agent 审计确认了复用 `NetworkEventReceiver` 的方案，并发现 `NetworkSender` 需扩展以支持发送 `LobbyReady`。

## Goals / Non-Goals

**Goals:**
- `NetworkEvent` 新增 `LobbyUpdate` 变体（复用 `NetworkEventReceiver`）
- `NetworkSender` 增加 `send_lobby_ready()` 方法（one-shot 槽位）
- `run_session()` 中 LobbyUpdate → push 到 `NetworkEventReceiver`
- `LobbyPhase` 新增 `Readying` 阶段
- Ready 按钮 + 玩家就绪列表 UI
- write task 在 drain loop 前检查 lobby_ready 槽位

**Non-Goals:**
- 不改 relay 协议（已完成）
- 不改地图选择 UI（下个 `lobby-flow-integration`）
- 不改 cancel 行为

## Decisions

### D1：复用 NetworkEventReceiver

`NetworkEvent` 新增 `LobbyUpdate` 变体，无需新通道。`run_session` 的 tokio read loop 中 push 到此 receiver。

### D2：NetworkSender 扩展

```rust
// 新增字段
lobby_ready: Arc<Mutex<Option<RelayClientMessage>>>,

pub fn send_lobby_ready(&self, player_id: u8) {
    *self.lobby_ready.lock().unwrap() = Some(RelayClientMessage::LobbyReady {...});
}
```

write task 在 `drain_all()` 前先检查并发送 lobby_ready 消息。

### D3：LobbyPhase 状态机

```rust
pub enum LobbyPhase {
    Connecting,  // TCP 连接中
    Connected,   // TCP 已通，等待 Ready
    Ready,       // 已按 Ready，等待其他玩家
    Failed(String), // 连接失败
}
```

## Risks

| Risk | Mitigation |
|------|-----------|
| NetworkSender 线程安全 | Arc<Mutex> 模式，已由 PlayerTickFrame 验证 |
| LobbyUpdate 时序 | lobby_update_system 读取 NetworkEventReceiver 前调用 |
| Cleanup  生命周期 | NetworkEventReceiver 在 Playing 后继续活跃（cleanup_playing 清理） |

## Post-Implementation Confirmation

三个子Agent 确认：
1. 跨线程通道（Arc<Mutex>）线程安全，无死锁
2. LobbyPhase 状态机构建完整
3. Cleanup 生命周期覆盖所有 Resource
4. 宪法合规，§11 7/7 通过

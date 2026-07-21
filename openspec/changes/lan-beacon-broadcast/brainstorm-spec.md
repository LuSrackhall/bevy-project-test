## Context

### 背景

当前 LAN 多人模式中，用户点击"创建房间"后，`SessionController::create_session()` 成功创建 Session，`ThreadRelayRuntime` 在后台线程启动 tokio runtime 并绑定 TCP 端口，但房间不在任何客户端的房间列表中显示。

### 根因

房间列表 `LanServers` 完全由 UDP 发现信标（beacon）驱动。`LanDiscoveryListener` 监听 `0.0.0.0:9876` 接收信标。但 `ThreadRelayRuntime::run_local_relay` 的占位实现只绑定了 TCP 端口后空转休眠，从未广播 UDP 信标，因此无人收到发现包，房间始终不出现。

### 当前实现

```rust
// crates/bevy_adapter/src/session_host/thread.rs
async fn run_local_relay(port_tx, seed: u64, max_players: u8, stop: &AtomicBool) {
    // 创建 RelayServer 状态机
    // 绑定 TCP 127.0.0.1:0
    // port_tx.send(Ok(actual_port))
    // 循环：每 100ms 检查 stop
    // ❌ 没有 UDP beacon 广播
}
```

## Goals / Non-Goals

**Goals：**

- 创建房间后，创建者的房间出现在本地房间列表中
- 同一 LAN 下的其他客户端也能通过 UDP 发现该房间
- 单机双窗口测试场景（`127.0.0.1:9876`）也能正常发现
- 信标内容包含完整的房间信息（房间名、地图、人数、状态、连接地址）
- all UDP errors 仅 log，不阻塞房间创建

**Non-Goals：**

- TCP 连接处理 / JoinGame 协议（属于 Change B）
- 自定义房间名输入加强（属于 #10）
- 房间等待页（属于 #11）
- relay crate 代码复用（属于 Change B）

## Decisions

### D1: UDP Socket 绑定 `0.0.0.0:{actual_port}`

必须绑定到 `0.0.0.0`（全接口）而非 `127.0.0.1`（回环），否则发送 `255.255.255.255` 广播将被回环接口过滤，跨机器发现完全失效。

TCP 监听仍保持在 `127.0.0.1:0`（仅本机回环连接，安全且 OS 自动分配端口）。

### D2: 强制 `set_broadcast(true)`

tokio `UdpSocket` 默认不允许发送广播。缺少此调用时 `send_to("255.255.255.255:9876")` 返回 `PermissionDenied`。

### D3: 信标集成到主循环（`tokio::select!`）

不 `tokio::spawn` 独立任务（避免 `&AtomicBool` 生命周期不满足 `'static`），而是用 `tokio::select!` 将 beacon 间隔与 stop 检查集成到同一个循环：

```rust
let mut beacon_interval = tokio::time::interval(Duration::from_secs(3));
loop {
    if stop.load(Ordering::Relaxed) { break; }
    tokio::select! {
        _ = beacon_interval.tick() => { /* 发送 beacon */ }
        _ = tokio::time::sleep(Duration::from_millis(100)) => {}
    }
}
```

### D4: 双目标广播

信标同时发往两个地址：

| 目标 | 作用 | 端口 |
|---|---|---|
| `255.255.255.255:9876` | LAN 广播（跨机器） | 9876 |
| `127.0.0.1:9876` | 本地回环（单机双窗口） | 9876 |

macOS 上 `255.255.255.255` 不经过回环接口，单机测试时必须通过 `127.0.0.1` 送达。

### D5: `port_tx.send` 在 UDP 初始化之前

TCP 绑定成功后就立即发送端口号，UDP socket 绑定放在之后。确保 `start()` 的 `port_rx.recv()` 不被 UDP 设置延迟。UDP 失败仅 log，完全不影响房间创建。

### D6: RelayId 由外部传入

```
start() 生成随机 RelayId → 传给 run_local_relay → 用于信标和 RelayHandle
```

保证信标中的 `relay_id` 与 `SessionController.current_relay_id()` 返回的值一致，防止去重失效。`run_local_relay` 签名改为 `(port_tx, relay_id: RelayId, room: &RoomMetadata, stop)`。

### D7: 所有 UDP 错误仅 log

UDP 绑定失败、编码失败、发送失败全部只做 `eprintln!` 或 `bevy::log::warn!`，绝不 panic 或返回错误。信标是尽力而为的服务。

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|---|---|---|
| `255.255.255.255` 被防火墙/路由器阻止 | LAN 发现失效 | 额外发 `127.0.0.1:9876` 至少保证本地；Change B 后可考虑 mDNS 作为备用 |
| macOS 网络栈对有限广播的处理差异 | 某些配置下跨机发现不可靠 | 属于 OS 级限制，文档记录；未来可升级到 mDNS |
| `current_players` 在 Change A 阶段始终为 0 | 房间显示 `0/2` 人 | 文档备注；Change B 实现 TCP accept 后从连接数动态获取 |
| 后台线程 panic 导致信标停止 | 已有房间可见性丢失 | port_tx.send 已在线程早期执行，房间创建本身已成功；panic 是编程错误应修复 |

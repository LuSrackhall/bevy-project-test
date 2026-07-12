## Context

当前联机模式下，客户端通过 `--relay <ip>:<port> --player-id <id>` CLI 参数手动连接 relay。局域网模式（#3）要求用户能点击"创建房间"一键启动本地 relay，无需接触 IP/端口。

已有基础设施：
- `relay/src/lib.rs`: `start_relay(port, seed, player_count)` — tokio async TCP relay
- `bevy_adapter::transport::spawn_network_client`: 线程 + tokio runtime 模式
- `bevy_adapter::discovery`: 刚完成的发现领域模型（#5），含 `RoomMetadata`、`RoomAdvertisement`
- `NeedsGameReset::Network`: 当前网络模式初始化路径

## Goals / Non-Goals

**Goals:**
- 定义 `RelayRuntime` trait：可替换的 relay 创建策略
- 定义 `RelayHandle` trait：运行中 relay 实例的句柄
- 定义 `Session` struct：`RoomMetadata` + `RelayHandle` 的组合
- 定义 `SessionController` struct：管理当前 `Session` 的生命周期
- 提供默认实现 `ThreadRelayRuntime`（spawn 线程 + tokio + `start_relay`）
- 在 `bevy_adapter` 中实现，不污染 `simulation`

**Non-Goals:**
- 子进程 relay 启动方式（ProcessRelayRuntime）
- Dedicated Relay 分配
- Host Migration
- 修改 relay 内部 tick/命令协议

## Decisions

### AD1: 接口设计

```rust
// bevy_adapter::session_host

/// Relay 创建策略。默认实现 ThreadRelayRuntime。
trait RelayRuntime {
    fn start(&mut self, room: &RoomMetadata)
        -> Result<Box<dyn RelayHandle>, RelayError>;
}

/// 运行中 Relay 实例的句柄。
trait RelayHandle {
    fn relay_id(&self) -> RelayId;
    fn endpoint(&self) -> SocketAddr;
    fn shutdown(self: Box<Self>) -> Result<(), RelayError>;
}

/// 一个 Session = 房间 + 正在运行的 Relay
struct Session {
    room: RoomMetadata,
    relay: Box<dyn RelayHandle>,
}

/// Session 生命周期控制器。
/// I1: 一个客户端同时最多管理一个 Session。
struct SessionController {
    runtime: Box<dyn RelayRuntime>,
    session: Option<Session>,
}
```

`RelayRuntime::stop()` 被删除——生命周期完全交给 `RelayHandle::shutdown()`，避免职责重复。

### AD2: endpoint 类型

使用 `std::net::SocketAddr`（`IpAddr` + `port`）。不预留 WebRTC 等非 Relay 传输层。

当前网络路线已锁定：
- LAN：TCP Relay
- 公网：Dedicated Relay（仍是 Relay）
- Web：WebSocket → Relay（浏览器不直连 TCP）
- WebRTC：暂不采用

### AD3: 默认实现

```rust
struct ThreadRelayRuntime;

impl RelayRuntime for ThreadRelayRuntime {
    fn start(&mut self, room: &RoomMetadata) -> Result<Box<dyn RelayHandle>, RelayError> {
        let (port_tx, port_rx) = std::sync::mpsc::channel();
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io().enable_time().build()?;
            rt.block_on(async {
                let listener = TcpListener::bind("127.0.0.1:0").await?;
                let actual_port = listener.local_addr()?.port();
                port_tx.send(Ok(actual_port)).ok();
                // ... accept loop (placeholder for #8)
            })
        });
        let actual_port = port_rx.recv()
            .map_err(|_| RelayError::StartFailed("Thread died".into()))??;
        Ok(Box::new(ThreadRelayHandle { endpoint: SocketAddr::from(([127,0,0,1], actual_port)), ... }))
    }
}
```

复用 `spawn_network_client` 的线程 + tokio 模式。

### AD4: RoomMetadata 来源

`SessionController` 创建 `Session` 时需要 `RoomMetadata`。数据来自用户在 UI 上选择的配置（地图、人数等）。`room_id` 和 `relay_id` 在启动时随机生成。

## Invariants

### I1: 单 Session

`SessionController.session` 是 `Option<Session>`，不是 `Vec<Session>`。一个客户端同时最多托管一个房间。多房间属于未来 Dedicated Relay 的职责。

### I2: SessionController 不维护运行期状态

`SessionController` 不主动修改 `RoomMetadata`。运行期状态（`current_players`、`game_state`）仅由 Relay 权威更新。`SessionController` 只负责生命周期管理。

## Risks / Trade-offs

- **[R1] 线程模式共享进程**：relay 与游戏同进程，一方崩溃双方都崩。当前阶段可接受，未来可切换到 ProcessRelayRuntime 或 Dedicated Relay。
- **[R2] RoomMetadata 创建时确定**：`room_id`、`map_id`、`max_players` 不可变，符合设计但限制了动态调整。
- **[R3] 不做 Host Migration**：房主掉线对局结束。MVP 接受此限制。

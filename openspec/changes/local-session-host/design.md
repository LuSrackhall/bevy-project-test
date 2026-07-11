## Context

局域网模式（#3）的子任务 #6。需要抽象 Session 生命周期管理层，让"创建房间"按钮能一键启动本地 relay。

详见 `brainstorm-spec.md` 中的设计决策 AD1-AD4 和不变量 I1-I2。

## Goals / Non-Goals

**Goals:**
- `bevy_adapter::session` 模块完整实现：traits + 默认 impl + 错误类型
- `ThreadRelayRuntime` 能正常启动/停止 relay
- `SessionController` 管理 `Option<Session>` 生命周期
- 单元测试覆盖正常路径和错误路径

**Non-Goals:**
- 不集成到 UI（#7 负责）
- 不修改 relay crate
- 不做断线重连

## Decisions

### 模块结构

```
bevy_adapter/src/session/
├── mod.rs          → 重新导出
├── controller.rs   → SessionController, Session
├── runtime.rs      → RelayRuntime trait, RelayHandle trait
├── error.rs        → RelayError
└── thread.rs       → ThreadRelayRuntime, ThreadRelayHandle
```

### ThreadRelayRuntime 实现

复用 `transport.rs` 的 `spawn_network_client` 模式：

```rust
impl RelayRuntime for ThreadRelayRuntime {
    fn start(&mut self, room: &RoomMetadata) -> Result<Box<dyn RelayHandle>, RelayError> {
        let (port_tx, port_rx) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io().enable_time().build()?;
            rt.block_on(async {
                // 绑定 :0 获取 OS 分配端口
                let listener = TcpListener::bind("127.0.0.1:0").await?;
                let actual_port = listener.local_addr()?.port();
                port_tx.send(actual_port).ok();
                // ... 复用现有 start_relay 逻辑
            })
        });

        let actual_port = port_rx.recv().map_err(|_| RelayError::StartFailed)?;
        Ok(Box::new(ThreadRelayHandle { stop, handle, relay_id, endpoint }))
    }
}
```

### SessionController API

```rust
impl SessionController {
    pub fn new(runtime: Box<dyn RelayRuntime>) -> Self;
    pub fn is_active(&self) -> bool;
    pub fn create_session(&mut self, room: RoomMetadata) -> Result<&Session, RelayError>;
    pub fn current_session(&self) -> Option<&Session>;
    pub fn destroy_session(&mut self) -> Result<(), RelayError>;
}
```

`create_session` 内部调用 `runtime.start()`，成功后将返回的 handle + room 组装成 `Session`。

### 与现有系统集成

`SessionController` 作为 Bevy Resource 注入：

```rust
commands.insert_resource(SessionController::new(Box::new(ThreadRelayRuntime)));
```

UI 层通过 ResMut<SessionController> 调用创建/销毁房间。

## Risks / Trade-offs

- **[R1] 线程同步**：relay 的 tokio runtime 在单独线程，端口通过 channel 传回。channel 断连视为启动失败。
- **[R2] stop flag 竞态**：`AtomicBool` 控制停止，relay 线程可能延迟退出。`thread::join` 确保退出完成。
- **[R3] start_relay 签名要求**：当前 `start_relay` 固定端口且启动即开始广播。改为 OS 分配端口后需先获取端口再启动广播。

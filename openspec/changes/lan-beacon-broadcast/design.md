## Context

当前 `ThreadRelayRuntime::run_local_relay` 在后台线程使用 tokio runtime 绑定 TCP 端口 0 后空转休眠。`lan-discovery` 规范要求 relay 广播 UDP 信标，但此占位实现完全缺失该能力。设计细节见 [brainstorm-spec.md](brainstorm-spec.md)（Context、Decisions、Risks 已在设计阶段的四轮 Agent 评估中充分讨论）。

## Goals / Non-Goals

**Goals：**
- 在 `run_local_relay` 的 tokio 循环中增加 UDP beacon 广播，每 3 秒发送 `LanDiscoveryPacket`
- 广播目标：`255.255.255.255:9876`（LAN） + `127.0.0.1:9876`（本地回环）
- UDP socket 绑定 `0.0.0.0:{TCP_port}`，启用广播
- 所有 UDP 错误仅 log，不 panic，不阻塞房间创建
- `RelayId` 由 `start()` 生成并传入线程，保证去重一致性

**Non-Goals：**
- TCP 连接处理、JoinGame 协议（Change B）

## Decisions

### D1: 集成到主循环而非 spawn 独立任务

`stop: Arc<AtomicBool>`（从 `&AtomicBool` 改为 `Arc`）以配合 tokio 任务生命周期。但为保持简单，使用 `tokio::select!` 将 beacon 间隔与 stop 检查集成到同一个循环，避免 spawn 的开销和关闭竞态。

```
let mut interval = tokio::time::interval(Duration::from_secs(3));
loop {
    if stop.load(Ordering::Relaxed) { break; }
    tokio::select! {
        _ = interval.tick() => { send_beacon() }
        _ = tokio::time::sleep(Duration::from_millis(100)) => {}
    }
}
```

### D2: `run_local_relay` 签名变更

```rust
// 旧
async fn run_local_relay(
    port_tx: &mpsc::Sender<Result<u16, RelayError>>,
    seed: u64,
    max_players: u8,
    stop: &AtomicBool,
)

// 新
async fn run_local_relay(
    port_tx: &mpsc::Sender<Result<u16, RelayError>>,
    relay_id: RelayId,
    room: &RoomMetadata,
    stop: &Arc<AtomicBool>,
)
```

### D3: UDP socket 与 TCP 端口解耦

TCP 绑定到 `127.0.0.1:0`（回环+OS分配），UDP 绑定到 `0.0.0.0:{actual_port}`（全接口）。两者共享同一端口号，但绑定到不同接口地址。macOS 上 TCP 和 UDP 是独立协议栈，端口复用合法。

### D4: 调用侧适配

`ThreadRelayRuntime::start()` 中将 `stop: Arc<AtomicBool>` 传给线程，生成 `RelayId` 后同时用于 `RunLocalRelay` 和 `ThreadRelayHandle`。

## Migration Plan

单步切换：重新编译即可。当前占位循环行为无外部依赖，替换后房间可见性提升。

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| UDP 防火墙/策略限制 broadcast | 日志记录便于排查；后续可升级为 mDNS |
| `255.255.255.255` 在 macOS 不到回环 | 额外发 `127.0.0.1:9876` |
| current_players 暂为 0 | 文档记录，Change B 修复 |

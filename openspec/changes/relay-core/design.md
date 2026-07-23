## Context

变更 B1 抽取共享 relay 运行时。高维设计见 [brainstorm-spec.md](brainstorm-spec.md)。

## Goals / Non-Goals

**Goals：**
- 抽取 `bevy_adapter::relay_core` 模块，包含完整 TCP accept + handle_client 逻辑
- ThreadRelayRuntime 调用 `run_relay()` 替代空转
- relay crate 调用 `run_relay()` 替代自实现
- `run_relay` 支持 `&AtomicBool` 停止信号

**Non-Goals：**
- UDP beacon 消除（Change B2）
- non-critical 死代码清理

## Decisions

### D1: relay_core.rs 模块结构

```rust
// bevy_adapter::relay_core (pub mod relay_core)
//
// 私有:
//   struct RelayCtx { server, clients }
//   async fn handle_client(ctx, stream)  ← 完整客户端处理
//   async fn relay_write(writer, msg)
//
// 公开:
//   pub struct RelayConfig { relay_id, seed, ... }
//   pub async fn run_relay(listener, config, stop)
//
// 迁移来源: relay/src/lib.rs 的 handle() + start_relay() 核心
// 注: next_player_id 字段不迁移（死代码），重复 JoinGame arm 不迁移
```

### D2: run_relay 实现

```rust
pub async fn run_relay(
    listener: TcpListener,
    config: RelayConfig,
    stop: &AtomicBool,
) {
    let now_ms = /* SystemTime */;
    let server = RelayServer::new(
        config.game_id, config.relay_id, config.ruleset_version,
        config.seed, config.map_spec_hash,
        (0..config.player_count).collect(),
        config.input_delay, now_ms,
    );
    let ctx = RelayCtx::new(server, config.player_count);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        eprintln!("[RELAY] Connect from {}", addr);
                        tokio::spawn(handle_client(ctx.clone(), stream));
                    }
                    Err(e) => {
                        eprintln!("[RELAY] Accept error: {}", e);
                        break;
                    }
                }
            }
            _ = /* 停止信号 */ => {
                break;
            }
        }
    }
}
```

停止信号实现：使用 `tokio::sync::futures::poll_fn` 或创建一个可在 `select!` 中使用的 future。最简单的方案是：

```rust
// 每 100ms 检查 stop
tokio::select! {
    result = listener.accept() => { ... }
    _ = tokio::time::sleep(Duration::from_millis(100)) => {
        if stop.load(Ordering::Relaxed) { break; }
    }
}
```

这与 Change A 的 beacon loop 模式一致。

### D3: handle_client 迁移

`relay/src/lib.rs` 的 `handle()` 函数整体迁移（~150 行），改动：

| 原代码 | 迁移后 | 说明 |
|---|---|---|
| RelayCtx 在 relay crate | RelayCtx 在 relay_core | 模块内迁移 |
| `handle(ctx, stream)` | `async fn handle_client(ctx, stream)` | 改名为 handle_client |
| 第 244 行死 `JoinGame => {}` | **删除** | 重复匹配臂 |
| `next_player_id: Mutex<u8>` | **删除** | 从未读取 |
| `relay_write` 私有 | `pub async fn relay_write` | 公开供外部使用 |

### D4: ThreadRelayRuntime 适配

```rust
async fn run_local_relay(port_tx, relay_id, room, stop) {
    // 1. RelayServer 创建（保持不变）
    // 2. TCP bind + port_tx.send（保持不变）
    // 3. UDP socket + beacon（保持不变，Change A 添加）
    // 4. relay_core::run_relay(listener, config, stop).await
    // 5. 保持 server / listener drop guard（beacon loop 退出后）
}
```

即：port_tx.send 后，将 listener 传给 relay_core，不再空转。

### D5: relay crate 适配

```rust
pub async fn start_relay(port, seed, player_count) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let config = RelayConfig { seed, player_count, .. };
    let stop = AtomicBool::new(false);
    relay_core::run_relay(listener, config, &stop).await;
    Ok(())
}
```

UDP beacon（原始 relay crate 版本）暂时保留，Change B2 清理。

## Migration Plan

单步切换：新增 relay_core.rs → 适配两处调用 → 删除 relay crate 中已迁移的代码。

1. 新建 `bevy_adapter/src/relay_core.rs`
2. 注册到 `bevy_adapter/src/lib.rs`
3. 修改 `thread.rs` → 改为调用 run_relay
4. 修改 `relay/src/lib.rs` → 改为调用 run_relay
5. 编译 + 测试验证

## Risks / Trade-offs

| 风险 | 缓解 |
|---|---|
| relay crate 的 handle() 整体迁移时可能漏分支 | 逐分支对照迁移，commit 前 code review |
| stop 信号使用 sleep 轮询而非 CancellationToken | 与 Change A 模式一致，可接受；后续可升级到 tokio-util 的 CancellationToken |
| relay 集成测试依赖 start_relay | 保留薄包装，测试无需迁移 |

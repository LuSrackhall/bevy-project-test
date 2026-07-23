## Context

### 背景

当前代码库中有两套 relay 运行时逻辑：

1. **`relay` crate (`crates/relay/src/lib.rs`)** — 完整实现：TCP bind/accept、`handle_client`（JoinGame → GameJoined → tick 收集/广播）、UDP beacon
2. **`ThreadRelayRuntime` (`crates/bevy_adapter/src/session_host/thread.rs`)** — 只有 UDP beacon，TCP 绑定后空转休眠

两者在 RelayServer 创建、handle_client 逻辑上存在大量重复（~80%），且行为不一致（relay crate 有完整的 accept 循环，ThreadRelayRuntime 没有）。这导致联机流程在"发现房间"后断裂——他人可以看见房间但无法加入。

### 根因

历史上 TCP accept + handle 逻辑先写在 `relay` crate 里供独立 CLI 使用。`ThreadRelayRuntime` 作为游戏进程内嵌版本长期是空壳待替换。Change A 添加了 UDP beacon 后，ThreadRelayRuntime 可广播房间但无连接处理能力。

### 当前结构

```
relay/src/lib.rs (完整)               bevy_adapter::session_host::thread (空壳)
┌────────────────────────┐           ┌─────────────────────────────┐
│ RelayServer 创建       │           │ RelayServer 创建            │
│ TCP bind (全接口)      │           │ TCP bind (127.0.0.1:0)     │
│ UDP beacon 🔴 (重复)  │           │ UDP beacon ✅              │
│ TCP accept ✅          │   重复    │ TCP accept ❌              │
│ handle_client ✅       │ ──────→   │ run_local_relay (空转休眠)  │
└────────────────────────┘           └─────────────────────────────┘
```

## Goals / Non-Goals

**Goals：**

- 将 relay crate 中的 TCP accept + handle_client 完整逻辑抽取到 `bevy_adapter::relay_core` 公共模块
- `ThreadRelayRuntime` 的循环从空休眠改为调用 `relay_core::run_relay()`
- `relay` crate 的 `start_relay()` 也改为调用 `relay_core::run_relay()`（薄包装）
- `run_relay()` 支持停止信号（`&AtomicBool`），使 ThreadRelayHandle::shutdown 正常工作
- 跨机联机首次可完整运行（发现房间 → 加入 → GameStarted → tick 广播）

**Non-Goals：**

- UDP beacon 重复消除（relay crate 中残留的 beacon 保留，属 Change B2 / #13）
- `relay` crate 中已知的死代码清理（第 244 行重复 JoinGame arm、RelayCtx 中 `next_player_id`）
- 非 relay 核心的基础设施重构

## Decisions

### D1: 共享核心位置 — `bevy_adapter::relay_core`

`relay` crate 已依赖 `bevy_adapter`（Cargo.toml），所以 `bevy_adapter` 中放公共模块不需要新增依赖。relay crate 通过 `bevy_adapter::relay_core::run_relay()` 调用。

如果新建独立 `relay-core` crate，需要协调类型归属（RelayServer、RelayClientMessage 等都在 bevy_adapter 定义），引入三方依赖，复杂度得不偿失。

### D2: `run_relay` 签名

```rust
pub async fn run_relay(
    listener: TcpListener,
    config: RelayConfig,
    stop: &AtomicBool,
)
```

- `listener`：已绑定的 TcpListener，端口策略由调用者决定（127.0.0.1:0 或 0.0.0.0:指定端口）
- `config`：静态配置结构体
- `stop`：停止信号，`tokio::select!` 与 `listener.accept()` 竞争

```rust
pub struct RelayConfig {
    pub relay_id: RelayId,
    pub game_id: u64,
    pub ruleset_version: u32,
    pub seed: u64,
    pub map_spec_hash: u64,
    pub player_count: u8,
    pub input_delay: u32,
}
```

### D3: Beacon 归属 — 本次不改

ThreadRelayRuntime 的 UDP beacon 保持在其 `run_local_relay` 中（Change A 添加）。`run_relay` 不处理 beacon。relay crate 的 beacon 也保持现有（Change B2 清理）。

在 ThreadRelayRuntime 中，线程循环改为：

```
1. TCP bind 127.0.0.1:0
2. port_tx.send(Ok(actual_port))   ← 不阻塞房间创建
3. 创建 UDP socket → 启动 beacon 循环（tokio::spawn 在同一个 runtime）
4. relay_core::run_relay(listener, config, &stop)   ← 阻塞直到 stop
```

### D4: relay crate 适配

`start_relay()` 成为薄包装：

```rust
pub async fn start_relay(port: u16, seed: u64, player_count: u8) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    let config = RelayConfig { seed, player_count, ... };
    let stop = AtomicBool::new(false);  // 不停止，由持有者控制
    bevy_adapter::relay_core::run_relay(listener, config, &stop).await;
    Ok(())
}
```

### D5: 测试兼容

relay crate 的集成测试（6 个测试）和 bevy_adapter 的 network_e2e 测试都直接调用 `relay::start_relay()`。保留 `start_relay` 为薄包装器后，测试无需迁移。

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|---|---|---|
| 迁移 handle_client 时遗漏消息分支 | 联机逻辑不完整 | handle 函数有 7 个清晰的消息分支，逐条对照迁移 |
| run_relay 与 ThreadRelayRuntime 的 beacon 循环交互 | beacon 和 accept 共享 tokio runtime | beacon 用 tokio::spawn 在 run_relay 前启动，不冲突 |
| relay crate 的 handle_client 有 dead code（重复 JoinGame arm） | 迁移后死代码转移到 bevy_adapter | 迁移时主动删除第 244 行死匹配臂 |
| relay crate 的 RelayCtx 有 dead field（next_player_id） | 无功能影响 | 迁移时删除 |

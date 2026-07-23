## Why

当前两套 relay 运行时逻辑（`relay` crate 的完整实现 与 `ThreadRelayRuntime` 的空壳）存在 ~80% 重复，且行为不一致。ThreadRelayRuntime 只广播 UDP beacon 但不 accept TCP 连接，导致联机流程在"发现房间"后断裂。需要抽取共享运行时，使两边共用同一份核心逻辑。

## What Changes

- 新增 `bevy_adapter::relay_core` 模块：`run_relay()` + `RelayConfig` + `relay_write()`
- `run_relay` 签名：`(listener: TcpListener, config: RelayConfig, stop: &AtomicBool)`
- `ThreadRelayRuntime` 的循环从空休眠改为调用 `relay_core::run_relay()`
- `relay::start_relay()` 改为调用 `relay_core::run_relay()` 的薄包装
- 迁移时清理已知死代码（relay crate 第 244 行重复 JoinGame arm、`next_player_id` 字段）
- **BREAKING**: `relay::start_relay()` 签名不变，但实现方式更改

## Capabilities

### New Capabilities
- `relay-core`: 共享 relay 运行时 — TCP accept + handle_client + tick 广播

### Modified Capabilities
- `relay-server`: 实现方式从独立实现变为 relay_core 薄包装

## Impact

| 范围 | 文件 | 说明 |
|---|---|---|
| 新增 | `crates/bevy_adapter/src/relay_core.rs` | 共享运行时模块 |
| 修改 | `crates/bevy_adapter/src/session_host/thread.rs` | 改为调用 run_relay() |
| 修改 | `crates/bevy_adapter/src/lib.rs` | 注册 relay_core 模块 |
| 修改 | `crates/relay/src/lib.rs` | 改为薄包装 |
| 删除 | relay crate (internal) | 死代码（重复 JoinGame arm + next_player_id） |
| 无 | Cargo.toml | 不新增依赖 |

## Why

局域网模式（#3）要求用户能点击"创建房间"一键启动本地 relay。当前只能通过 CLI 参数手动指定 `--relay <ip>:<port>`，普通用户不可用。需要抽象的 Session 生命周期管理层，将 relay 启动/停止封装为可替换的策略。

## What Changes

- 新增 `bevy_adapter::session_host` 模块，定义：
  - `RelayRuntime` trait：可替换的 relay 创建策略
  - `RelayHandle` trait：运行中 relay 实例的句柄
  - `RelayError`：统一的 relay 错误类型
  - `Session` struct：`RoomMetadata` + `RelayHandle` 的组合
  - `SessionController` struct：管理当前 `Session` 的创建与销毁
  - `ThreadRelayRuntime`：默认实现，spawn 线程 + tokio + `start_relay()`
  - `ThreadRelayHandle`：对应线程模式的 handle 实现
- 新增 `RelayId` 类型（`u64`），与 `discovery::RelayId` 对齐（后续可用同一类型）
- 不修改现有 `relay` crate
- 不修改 `simulation`

## Capabilities

### New Capabilities
- `session-controller`: 可替换的 RelayRuntime 策略 + SessionController 生命周期管理。提供 ThreadRelayRuntime 默认实现。

### Modified Capabilities
<!-- 无现有 spec 变更 -->

## Impact

- 新增 `bevy_adapter::session_host` 模块，4 个 trait/struct + 1 个默认实现
- 新增错误类型 `RelayError`
- 依赖：`discovery` 模块（`RoomMetadata`）、`relay` crate（`start_relay`）
- 不改变现有网络协议或数据面

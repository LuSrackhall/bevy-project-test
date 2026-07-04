## Why

v0.4.0 已实现联机基础设施（Relay-backed deterministic lockstep），但游戏入口仅支持单人/回放模式。`CommandSource::Network` 从未被实例化，玩家无法通过 UI 进入联机对战。需要将 Network 入口接入游戏 + 统一所有模式的初始化管道。

## What Changes

- 新增 `SessionBootstrap` 层：UI → GameIntent → SessionConfig → dispatch → wire 的标准化初始化管道
- 新增 `NetworkSessionManager`：建立 relay 连接、完成握手、启动 NetworkCommandSource
- 新增主菜单 Network 入口：输入 relay 地址 + 玩家数量 → 连接 → 开始游戏
- 新增 `GameIntent` enum（UI 层）：Single / Network / Replay 三种启动意图
- 新增 `SessionConfig` / `SessionArtifacts`：初始化配置与产物分离，wire() 统一组装
- 新增 `BootstrapPhase`：Driver 的生命周期状态（Init → Wired → Active）
- 新增 `resolve_intent()`：GameIntent → SessionConfig 的纯转换（render_view 层）
- 新增 `SessionInitializer` 模块：`single::initialize()` / `replay::initialize()` / `network::initialize()`
- 新增 `connect_and_handshake()`：transport 层增加 GameJoined 回传 channel（约 10 行）
- 新增 5 项不变量（P1-P10）：Atomicity、Ownership、GameJoined 封闭等
- 新增 e2e 集成测试验证 Network → Replay 录制全链路
- 不修改 `simulation` crate（宪法 §1.2.7 零感知原则）
- 不修改 relay 协议（v0.4.0 已冻结）

## Capabilities

### New Capabilities
- `session-bootstrap`: GameIntent → SessionConfig → dispatch → wire 管道
- `network-start-ui`: 主菜单联机入口（relay 地址 + 玩家数量输入）
- `session-initializers`: single / replay / network 三种模式的初始化模块
- `bootstrap-phase`: Driver 生命周期状态管理

### Modified Capabilities
- (无) — 不修改 Simulation 层

## Impact

- `crates/bevy_adapter/src/session/bootstrap.rs`：新增 dispatch() + wire() + SessionArtifacts + BootstrapPhase
- `crates/bevy_adapter/src/session/single.rs`：新增 single::initialize()
- `crates/bevy_adapter/src/session/replay.rs`：新增 replay::initialize()
- `crates/bevy_adapter/src/session/network.rs`：新增 network::initialize() + connect_and_handshake()
- `crates/render_view/src/session.rs`：新增 GameIntent + resolve_intent()
- `crates/render_view/src/ui/network_panel.rs`：新增联机输入面板
- `crates/render_view/src/ui/menu.rs`：主菜单添加"联机"区域
- `crates/bevy_adapter/src/driver.rs`：新增 BootstrapPhase enum 到 SimulationDriver
- `crates/bevy_adapter/src/transport.rs`：新增 ~10 行 GameJoined 通道
- 外部 crate 无感知：presentation、simulation、relay 不修改

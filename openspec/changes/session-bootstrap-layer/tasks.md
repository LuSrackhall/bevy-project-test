## 1. 项目结构

- [x] 1.1 创建 `crates/bevy_adapter/src/session/` 目录及 mod.rs
- [x] 1.2 创建 `crates/bevy_adapter/src/session/bootstrap.rs`（dispatch, wire, SessionArtifacts, BootstrapPhase）
- [x] 1.3 创建 `crates/bevy_adapter/src/session/single.rs`
- [x] 1.4 创建 `crates/bevy_adapter/src/session/replay.rs`
- [x] 1.5 创建 `crates/bevy_adapter/src/session/network.rs`（NetworkBootstrapResult, connect_and_handshake）
- [x] 1.6 创建 `crates/render_view/src/session.rs`（GameIntent, resolve_intent）

## 2. 数据结构定义

- [ ] 2.1 定义 `GameIntent` enum（Single/Replay/Network，在 render_view）
- [ ] 2.2 定义 `SessionConfig` + `SessionMode`（在 bevy_adapter）
- [ ] 2.3 定义 `SessionArtifacts` enum（Live/Replay/Network）
- [ ] 2.4 定义 `NetworkBootstrapResult` + `TransportResources`
- [ ] 2.5 定义 `BootstrapPhase`（Init/Wired/Active）
- [ ] 2.6 新增 `bootstrap_phase: BootstrapPhase` 到 SimulationDriver

## 3. 初始化器实现

- [ ] 3.1 实现 `session::single::initialize()`（无参数，返回 ()）
- [ ] 3.2 实现 `session::replay::initialize()`（加载 ReplayFile）
- [ ] 3.3 实现 `session::network::initialize()`（connect_and_handshake）
- [ ] 3.4 实现 `connect_and_handshake()`（spawn_network_client_with_game_joined + recv_timeout + 错误清理）

## 4. Bootstrap 管道

- [ ] 4.1 实现 `resolve_intent()`（GameIntent → SessionConfig，render_view）
- [ ] 4.2 实现 `dispatch()`（根据 SessionMode 调用对应 initializer）
- [ ] 4.3 实现 `wire()`（按 artifact 类型构造 CommandSource + 注册资源 + 固定写入顺序）
- [ ] 4.4 实现 `SessionArtifacts::D4.1` move-only 约束（无 Clone, 无 Arc）
- [ ] 4.5 实现 BootstrapPhase 重入守卫（检查 phase == Init）
- [ ] 4.6 实现 P10：commit 顺序（init_world → recorder → resources → driver.source → phase = Wired）
- [ ] 4.7 实现 transition: check_wired system（Wired → Active）

## 5. UI 入口

- [ ] 5.1 主菜单添加"联机"区域（relay 地址输入 + 玩家数量选择 + 开始按钮）
- [ ] 5.2 点击 Start 时产生 GameIntent::Network
- [ ] 5.3 UI 显示 "SessionConnecting" 状态（bootstrap 期间）
- [ ] 5.4 bootstrap 完成后进入 Playing 状态

## 6. transport 适配

- [ ] 6.1 transport.rs 新增 `spawn_network_client_with_game_joined`（含 GameJoined 通道）
- [ ] 6.2 transport.rs 新增错误清理（失败时 abort tokio 线程）

## 7. 测试

- [ ] 7.1 单元测试：resolve_intent 各项场景
- [ ] 7.2 单元测试：dispatch → wire 管道
- [ ] 7.3 单元测试：BootstrapPhase 重入守卫
- [ ] 7.4 单元测试：connect_and_handshake 失败清理
- [ ] 7.5 回归测试：cargo test -p bevy_adapter 全部通过
- [ ] 7.6 回归测试：cargo test -p relay 全部通过

## 8. 文档

- [ ] 8.1 更新 `docs/engineering/command-pipeline-guide.md`（新增 SessionBootstrap 流程）

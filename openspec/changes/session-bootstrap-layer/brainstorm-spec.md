# Session Bootstrap Layer — UI → GameIntent → Driver 入口设计

> 变更名：session-bootstrap-layer
> 关联：宪法 §1.2.7、§2.5.4；v0.4.0 network-command-stream；CLAUDE.md (render_view UI 准则)

---

## Context

v0.4.0 已合并，联机基础设施（Relay-backed deterministic lockstep）就位。但游戏入口只支持单人/回放模式，`CommandSource::Network` 从未被实例化。玩家无法通过 UI 进入联机对战。

当前入口流：

```
main_menu (MapSizeBtn → NewGame)
  ↓
reset_game_system (init simulation + driver)
  ↓
Playing (simulation_driver_system)
```

需要新增一条 "Network" 路径，将 UI 输入（relay 地址、玩家数量）映射为 `CommandSource::Network`。

系统已具备的基础设施：
- relay TCP server（crates/relay/）
- transport.rs（跨线程 bridge + Bevy poll/flush systems）
- NetworkCommandSource + CommandSource::Network 变体

---

## Goals / Non-Goals

**Goals:**
1. 主菜单增加联机入口——输入 relay 地址；对局参数（map_size 等）由 Relay 协商确定。
2. 新增 GameIntent → SessionConfig → CommandSource 管道
3. 联机模式下通过 connect_and_handshake() 建立连接、分配 player_id、启动 tick loop
4. 联机对局自动录制回放（复用 ReplayRecorder）
5. resolve_intent() 作为纯数据转换层，不产生副作用
6. SessionBootstrap 作为唯一副作用函数（one-shot，guarded）

**Non-Goals:**
- 不做排位/匹配/房间/大厅系统
- 不做 Observer/观战模式
- 不修改 relay 协议（v0.4.0 已冻结）
- 不修改 Simulation 层
- 不修改 driver 的 tick 循环
- 不做玩家账户/认证

---

## Decisions

### D1: 分层架构（三域 + 握手协议隔离）

```
UI (render_view)
  ↓ GameIntent ← 纯 intent，无 I/O 无状态机
resolve_intent() (pure data transform, 在 render_view 中)
  ↓ SessionConfig ← init-only 数据，不进入 tick path
SessionBootstrap
    ├─ connect (TCP)
    ├─ handshake (JoinGame → GameJoined)  ← 握手协议，非运行时
    └─ create Driver + NetworkCommandSource
  ↓
Playing
  ↓
transport runtime (TickBatch, Disconnect, ...)
```

**三域生命周期 + 握手协议隔离：**

| 域 | 范围 | 职责 |
|----|------|------|
| **初始化域** | UI → Intent → Resolver | 纯数据映射 |
| **握手协议域** | TCP connect → JoinGame → GameJoined | 建立网络会话，产出 player_id |
| **运行时域** | Playing → transport → tick loop | TickBatch、Disconnect 等游戏事件 |

生命周期分界：**在 `driver.source = CommandSource::Network(...)` 之前发生的网络交互（JoinGame/GameJoined）属于握手协议域；之后发生的（TickBatch、Disconnect）属于运行时域。**

**关键约束：SessionBootstrap 不消费或等待运行时域事件（TickBatch、Disconnect、Reconnect）。** 但可以等待握手协议事件（GameJoined），因为握手协议在 driver.source 赋值之前完成。

- **GameIntent** = UI → 逻辑层的边界，属于 render_view
- **resolve_intent()** = 纯数据转换，属于 render_view。无 I/O 无副作用
- **SessionBootstrap** = bevy_adapter 服务函数，被 `reset_game_system` 调用。唯一的副作用入口

### D2: 数据结构与初始化产物

```rust
// render_view 层

/// UI → 逻辑层的纯意图描述。类型属于 UI 层（符合 UI CLAUDE.md:
/// Widget 产生 semantic event，不持有业务状态）。
pub enum GameIntent {
    Single { map_size: MapSize },
    Replay { path: PathBuf },
    Network {
        relay_addr: String,
        player_count: u8,
    },
}
```

```rust
// bevy_adapter/src/session.rs

/// Session 配置，由 resolve_intent() 从 GameIntent 转换而来。
/// 生命周期：bootstrap-scoped。bootstrap 完成后必须释放（must not be retained after bootstrap）。
pub struct SessionConfig {
    pub mode: SessionMode,
}

pub enum SessionMode {
    Single { map_size: MapSize },
    Replay { path: PathBuf },
    Network { relay_addr: String, player_count: u8 },
}

/// 初始化产物——enum 表达不同模式的产物形态。
/// 非 Option——每个变体精确描述该模式的资源集合。
pub enum SessionArtifacts {
    Live,
    Replay { replay: ReplayFile },
    Network(NetworkBootstrapResult),
}

/// Network initializer 返回的握手结果。
/// 不含 CommandSource——由 wire() 统一构造。
pub struct NetworkBootstrapResult {
    pub player_id: u8,
    pub receiver: NetworkReceiver,
    pub sender: NetworkSender,
    pub handle: NetworkClientHandle,
}

/// Initializer 创建的传输层资源。
/// 由 wire() 注册为 Bevy Resources。
pub struct TransportResources {
    pub receiver: NetworkReceiver,
    pub sender: NetworkSender,
    pub handle: NetworkClientHandle,
}
```

### D3: resolve_intent（纯函数，在 render_view 中）

```rust
// render_view/src/session.rs

/// GameIntent 是 UI 层类型，resolve_intent() 是翻译函数。
/// render_view 已依赖 bevy_adapter（用于 SessionConfig），
/// 因此翻译放在 render_view 不会产生反向依赖。
pub fn resolve_intent(intent: GameIntent) -> SessionConfig {
    match intent {
        GameIntent::Single { map_size } => SessionConfig {
            mode: SessionMode::Single { map_size },
        },
        GameIntent::Replay { path } => SessionConfig {
            mode: SessionMode::Replay { path },
        },
        GameIntent::Network { relay_addr, player_count } => SessionConfig {
            mode: SessionMode::Network { relay_addr, player_count },
        },
    }
}
```

语义翻译属于目标层（adapter），不属于源层（UI）。`GameIntent` 是 UI semantic，`SessionConfig` 是 driver semantic。转换逻辑在 adapter 中，UI 只产生 intent，不触碰 adapter 的初始化协议。

### D4: 模块化 Initializer（非 trait）

对于 3 种 SessionMode，直接用模块级函数而非 trait，等模式扩展到 5-6 个以上再引入抽象。

```rust
// bevy_adapter/src/session/network.rs

/// NetworkInitializer 只返回握手结果，不构造 CommandSource。
/// CommandSource 由 dispatch() 统一构建。
pub struct NetworkBootstrapResult {
    pub player_id: u8,
    pub receiver: NetworkReceiver,
    pub sender: NetworkSender,
    pub handle: NetworkClientHandle,
}

/// 建立连接、完成握手、返回 bootstrap facts。
/// I/O 层，唯一 side effect。
pub fn initialize(cfg: &SessionConfig) -> Result<NetworkBootstrapResult, String> {
    let relay_addr = match &cfg.mode {
        SessionMode::Network { relay_addr, .. } => relay_addr,
        _ => return Err("Not a Network session".into()),
    };
    let player_id = connect_and_handshake(relay_addr)?;
    Ok(NetworkBootstrapResult { player_id, receiver: ..., sender: ..., handle: ... })
}
```

```rust
// bevy_adapter/src/session/replay.rs

pub fn initialize(cfg: &SessionConfig) -> Result<ReplayFile, String> {
    let path = match &cfg.mode {
        SessionMode::Replay { path } => path,
        _ => return Err("Not a Replay session".into()),
    };
    let replay = load_replay(path)?;
    Ok(replay)
}
```

```rust
// bevy_adapter/src/session/single.rs

pub fn initialize() -> Result<(), String> {
    Ok(()) // LiveCommandSource 无参数
}
```

**返回值对照：**

| 模式 | initialize 返回 | dispatch 组合 |
|------|----------------|---------------|
| Single | `()` | `SessionArtifacts::Live` |
| Replay | `ReplayFile` | `SessionArtifacts::Replay { replay }` |
| Network | `NetworkBootstrapResult` | `SessionArtifacts::Network(result)` |

#### dispatch（registry）

```rust
fn dispatch(config: &SessionConfig) -> Result<SessionArtifacts, String> {
    let artifacts = match &config.mode {
        SessionMode::Single { .. } => {
            session::single::initialize()?;
            SessionArtifacts::Live
        }
        SessionMode::Replay { .. } => {
            let replay = session::replay::initialize(&config)?;
            SessionArtifacts::Replay { replay }
        }
        SessionMode::Network { .. } => {
            let result = session::network::initialize(&config)?;
            SessionArtifacts::Network(result)
        }
    };
    Ok(artifacts)
}
```

#### wire（CommanSource 构造 + 资源注册）

`wire()` 按 artifact 类型分支，构造对应的 CommanSource 并注册资源。此分支是生命周期分派（artifact type dispatch），不是业务判断。

```rust
fn wire(artifacts: SessionArtifacts, ctx: &mut InitCtx) {
    match artifacts {
        SessionArtifacts::Live => {
            ctx.driver.source = CommanSource::Live(LiveCommanSource);
        }
        SessionArtifacts::Replay { replay } => {
            ctx.driver.source = CommanSource::Replay(ReplayCommanSource { replay });
        }
        SessionArtifacts::Network(result) => {
            let ns = NetworkCommanSource::new(
                1, result.player_id, 3,
            );
            ctx.driver.source = CommanSource::Network(ns);
            ctx.insert_resource(TransportResources {
                receiver: result.receiver,
                sender: result.sender,
                handle: result.handle,
            });
        }
    }
    ctx.driver.phase = BootstrapPhase::Wired;
}
```

### D4.1 SessionArtifacts Ownership（Move-Only）

`SessionArtifacts` 是初始化阶段的一次性所有权对象（one-shot ownership），**必须由 `wire()` 以 move 方式消费**：

- Initializer 创建 → **ownership 转移到 dispatch**
- dispatch → **move 到 wire**
- wire 分解 → **分配到 Driver、World、Resources**
- wire 完成后 → **`SessionArtifacts` 立即 drop，不得进入 runtime**

因此：
- 必须通过 move 传递，不允许 `Clone`
- 不允许 `Rc`/`Arc`（`TransportResources` 内部的 `Arc` 是 bridge 实现细节，不属于 artifact 的共享）
- 不允许跨 runtime 生命周期保存
- **wire() 不得调用任何 initializer**——wire 只消费 `SessionArtifacts`（由 dispatch + initializeer 完全构造后传入），不重新请求 I/O 或构造新对象。违反此约束会破坏 `prepare → validate → commit` 的原子性保证。

重入 bootstrap 会导致：双 driver source overwrite、transport resource leak（多 receiver/sender instance）。BootstrapPhase + D4.1 的 single-consumption semantic 共同防止此问题。

此原则作用于 session-bootstrap-layer 变更引入的抽象，非全局宪法。
dispatch 和 wire 的实现见 D4 中的定义（dispatch = registry, wire = 对象图装配）。

### D5: InitCtx — wiring 上下文

```rust
/// SessionBootstrap::wire() 的上下文。封装 bootstrap 阶段需要修改的系统资源。
pub struct InitCtx<'a> {
    pub driver: &'a mut SimulationDriver,
    pub commands: Commands<'a, 'a>,
    pub world: &'a mut SimulationWorld,
    pub recorder: &'a mut ReplayRecorder,
    pub cmd_buf: &'a mut CommandBuffer,
    pub tick_clock: &'a mut TickClock,
    pub map_size: MapSize,
    pub seed: u64,
}
```

`wire()` 末尾设 `driver.phase = BootstrapPhase::Wired`（立即写入，非 deferred ECS Commands）。

### D5.1 wire() invariant

`wire()` 本质是对象图装配阶段（object graph hydration step），不是逻辑函数。只允许执行三类操作：

| 允许的操作 | 示例 |
|-----------|------|
| **assignment** | `driver.source = ...`（由 wire 根据 artifact 类型构造） |
| **registration** | `insert_resource(transport.receiver)` |
| **deterministic initialization** | `init_world(ctx)`, `setup_recorder(ctx)` |

**禁止：**
- 协议处理（解析或依赖 GameCommand）
- I/O（文件读写、网络请求）
- 基于业务语义的决策（依据游戏规则做条件分支）
- **允许：** 按 `SessionArtifacts` 变体的分支（`match artifacts { Live / Replay / Network }`）——这属于生命周期分派（对象图装配），不是业务判断

此约束防止 `wire()` 退化为"第二个 `reset_game_system`"。

**实现层标记：** Bevy `Commands` 是 deferred 的（frame boundary commit）。`wire()` 插入的 Resources（如 `TransportResources`）在下一帧的 system 中才能可见。因此任何在 bootstrap 后立即发生的操作不得依赖当前帧内就绪的 transport resource。

### D6: connect_and_handshake — 传输层适配

现有传输层（transport.rs）的工作方式：

- `spawn_network_client()` 在后台 tokio 线程上启动 TCP 连接
- relay 在 TCP accept 后**立即发送 `GameJoined`**（无需客户端先发送 JoinGame）
- transport 的异步读取循环 (`run_client`) 收到 `GameJoined`，但当前只打印，不回传 player_id

`connect_and_handshake()` 需要一个通道把 GameJoined 中的 player_id 从 tokio 线程带回 caller：

```rust
// transport.rs 的改动：为 spawn_network_client 增加一个参数
// 用于接收跨线程的 GameJoined 结果
pub fn spawn_network_client_with_game_joined(
    relay_addr: String, game_id: u64, ruleset_version: u32,
) -> (mpsc::Receiver<u8>, NetworkReceiver, NetworkSender, NetworkClientHandle)
//                                          ↑ 收到 GameJoined 后发送 player_id

// connect_and_handshake 包装：
pub fn connect_and_handshake(
    relay_addr: &str,
) -> Result<(u8, NetworkReceiver, NetworkSender, NetworkClientHandle), String> {
    let (game_joined_rx, rx, tx, handle) =
        spawn_network_client_with_game_joined(relay_addr.to_string(), 1, 1);
    let player_id = game_joined_rx
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "Handshake timeout".to_string())?;
    Ok((player_id, rx, tx, handle))
}
```

transport.rs 改动量：约 10 行（新增 `spawn_network_client_with_game_joined` + `run_client` 中的 `game_joined_tx.send()`）。relay 协议不变。

**实现说明：** 当前 `connect_and_handshake()` 采用同步阻塞实现（`recv_timeout(5s)`），适用于 Phase 1 bootstrap。未来可演进为异步 bootstrap 状态机（`Connecting → WaitingHandshake → Ready`），以支持 reconnect、hot join 等场景。不要在文档中将同步阻塞固化为架构不变性。

**错误清理约束：** `connect_and_handshake()` 超时或失败时，必须确保 tokio 线程已停止、TCP 连接已关闭。失败路径上残留的后台线程会导致 ghost client（已连接但未完成 bootstrap 的客户端）。实现方式：`NetworkClientHandle` 在失败时调用 `handle.abort()`（tokio JoinHandle::abort）或通过 drop guard 自动清理。

**P4/P8 约束：`GameJoined` 是 session-initialization-only barrier，不是 gameplay event。** 它只用于 session creation，**不得代表对已有 session 的重新进入（re-entry）**。reconnect 场景下需通过独立的 `ReconnectResponse` + `ResyncCompleted` 路径处理。`GameJoined` 一旦被用于 bootstrap，其语义即封闭——不可在 runtime 中重新发送或复用。

### D7: BootstrapPhase（唯一生命周期状态）

Bootstrap 重入守卫通过 `BootstrapPhase` 实现：`wire()` 入口检查 `driver.phase == Init`，已非 Init 则跳过。

#### BootstrapPhase 定义

```rust
/// Driver 的生命周期状态。由 bootstrap 控制，runtime 只读。
pub enum BootstrapPhase {
    /// 初始状态。bootstrap 进入前或已结束。
    Init,
    /// wire() 完成，transport resources 已通过 deferred Commands 插入。
    /// 下一帧 check_wired 系统推进到 Active。
    /// 属于 implementation detail——因 Bevy Commands 是 deferred 的。
    Wired,
    /// Tick loop 可执行。wire() 完成且 resources 已可见。
    Active,
}
```

`wire()` 末尾设 `driver.phase = Wired`（立即写入）。下一帧由 `check_wired` 系统推进到 `Active`。`simulation_driver_system` 检查 `driver.phase != Active` 则跳过 tick。

```rust
fn simulation_driver_system(...) {
    if driver.phase != BootstrapPhase::Active { return; }
    // tick loop...
}
```

**宪法对齐：** 消除了对 ECS resource timing 的依赖（§2.5.5 Scheduler 域盲）。`driver.phase` 由 driver 内部状态管理。

**P10：Bootstrap Atomicity (prepare → validate → commit)。** Bootstrap 必须是原子的。具体结构：

```text
prepare  (initialize, I/O, assembly of SessionArtifacts)
    ↓
validate (all artifacts ready, transport connected, files loaded)
    ↓
commit   (wire: mutate Driver/World/Resources, all writes guaranteed to succeed)
```

- prepare 阶段：做所有可能失败的事（网络、文件、I/O）。**不修改 Driver/World/Resources。**
- validate 阶段：确认所有 artifacts 完整。失败则返回 Err（phase 保持 Init）。
- commit 阶段：将 artifacts 写入系统。**此阶段应保证不失败**（因为所有可变操作已被 prepare 验证）。
- **commit 写入顺序固定**：`init_world() → setup_recorder() → insert_resource() → driver.source → driver.phase = Wired`。`driver.source` 和 `driver.phase` 是最后一步——写入前 Driver 仍处于 `Init`，写入后系统才被视为 Active。这个顺序确保不会有"source 已切但 world 未 init"的半初始化帧。

不允许出现 `phase != Init` 但部分字段未初始化或系统处于半修改状态的情况。

**隐含前提（P3）：TransportResources are session-scoped exclusive handles。** transport sender/receiver 不能跨 session reuse，否则 relay 会出现 ghost client。D4.1 的 exactly-once consumption 保证这一点——wire() 注册后 Artifacts 被释放，transport 资源绑定到当前 session 生命周期。

### D8: 辅助函数与参数来源

- **`game_id`**：当前所有 relay 会话使用 `game_id = 1`。未来由 relay 在 handshake 中下发（预留字段）。
- **`load_replay(path)`**：从文件路径加载 `ReplayFile`。已有 `ReplayFile::from_ron()` / `ReplayFile::open()` 可用。路径由 SessionConfig.mode 携带。
- **`init_world(ctx)`** / **`setup_recorder(ctx)`**：现有 `reset_game_system` 中的世界初始化和录制配置逻辑。

### D9: Bootstrap 是唯一允许构造 CommandSource 的入口

> **S1：SessionBootstrap 是唯一允许构造 `CommandSource`、创建 `NetworkClientHandle`、切换 Driver Source 的入口；任何其他系统不得直接修改 Driver 的 `CommandSource`。**

防止 debug 系统、UI 系统或工具系统直接替换 `driver.source`，破坏初始化边界。

**P7 约束：`driver.source` mutation is exclusive to bootstrap phase (pre-world schedule)。** 运行时（Playing 状态）禁止任何 ECS system 写 `driver.source`，包括 replay restore、debug override、rollback。所有 source 切换必须通过 `SessionBootstrap` 重新进入，回到 init 域再执行。这不只是权限约束，更是生命周期边界——任何 runtime 写 driver.source 都会导致 tick skew 或 replay divergence。

**隐含前提（P2）：`driver.source` 替换只能在 pre-tick 阶段发生（CommandSource swap is only valid in pre-tick phase boundary）。** 进入 Playing 状态后（tick loop 已运行），不应切换 source。未来的 pause → reload session 或 replay seek → restart 路径也必须遵守此边界，否则 Network/Replay 混用会出现 tick skew。

### D10: 对局参数的来源

- Network mode 的 **seed** 来自 relay 侧，不是本机随机生成
- Network mode 的 **map_size** 由 relay 协商确认，UI 不提供地图选择（UI 仅输入 relay 地址和玩家数量）
- Single/Replay 模式的 seed 和 map_size 逻辑不变（当前是 UI 选择 + 本机随机 seed）

---

## Risks / Trade-offs

- **[reset_game_system 过载]** → 拆为 intent + resolver + bootstrap，每层单一职责
- **[map_size 双源]** → Network mode 下 relay authoritative，UI 值仅作为 hint
- **[bootstrap 重入]** → BootstrapPhase 守卫（phase != Init 则跳过），一次性
- **[SessionConfig lifecycle]** → bootstrap 调用后即失效，不得在 runtime tick 路径中访问
- **[connect_and_handshake vs transport protocol]** → transport.rs 有约 10 行的适配改动（GameJoined 通道），协议本身不变。设计中已明确这对组改动
- **[handshake 同步阻塞]** → `recv_timeout(5s)` 在 bootstrap 线程同步阻塞。Network mode 下 UI 在连接期间会短暂冻结。**UI 契约：bootstrap 触发前必须显示 "SessionConnecting" 等连接状态指示，否则用户会认为游戏卡死。** 此阻塞在 RTS lockstep 设计中可接受（初始化属于一次性延迟，不影响运行时 tick）
- **[SessionMode 扩展压力]** → 当前 Single / Replay / Network 三种模式通过 `dispatch()` match 管理。未来扩展至 Spectator / Reconnect / Hot join / AI-only 等模式时，`dispatch()` 可能膨胀为 god match。推荐演进路径：每个模式返回自己的 artifact 类型（`NetworkArtifacts`、`ReplayArtifacts`），通过 `Into<SessionArtifacts>` 转换，dispatch 退化为纯 registry
- **[InitCtx 膨胀]** → 当前 `InitCtx` 约 8 个字段，未来可能继续增长。演进策略：超 12 个或出现自然分组时拆分为 `DriverInit`、`WorldInit`、`TransportInit` 等细粒度上下文类型
- **[ReplayRecorder 耦合]** → 当前 `InitCtx` 中包含 `recorder` 作为固定依赖。未来 headless、benchmark、server 等模式可能不需要录制。可演进为 bootstrap hook（`on_session_ready: Box<dyn FnOnce(&mut InitCtx)>`），使 recorder 成为可插拔组件
- **[架构等级]** → 当前设计达到 Level 4（Explicit Lifecycle Architecture）：lifecycle 显式分层、side effect 单入口、runtime/init/protocol 分域、ECS 资源不参与 control flow。已达到 Level 5 的核心要求（`bootstrap_phase` 消除 ECS timing dependency）。尚缺的：handshake 仍为同步阻塞。后续可沿三条路径演进：A. 时间模型收敛（已完成 BootstrapPhase）→ B. 网络 bootstrap async state machine → C. ECS-free bootstrap layer

---

## 实现路径

| 文件 | 操作 |
|------|------|
| `crates/render_view/src/ui/network_panel.rs` | **新文件** — 联机输入面板 UI（relay 地址 + player_count） |
| `crates/render_view/src/session.rs` | **新文件** — GameIntent enum + resolve_intent() 纯函数 |
| `crates/bevy_adapter/src/transport.rs` | **修改** — 新增 `spawn_network_client_with_game_joined`，约 10 行 |
| `crates/bevy_adapter/src/session.rs` | **新文件** — GameIntent, SessionConfig, SessionMode, resolve_intent |
| `crates/bevy_adapter/src/session/single.rs` | **新文件** — single::initialize() |
| `crates/bevy_adapter/src/session/replay.rs` | **新文件** — replay::initialize() |
| `crates/bevy_adapter/src/session/network.rs` | **新文件** — NetworkBootstrapResult, network::initialize(), connect_and_handshake |
| `crates/bevy_adapter/src/session/bootstrap.rs` | **新文件** — dispatch(), wire(), SessionArtifacts, TransportResources, BootstrapPhase |
| `crates/bevy_adapter/src/lib.rs` | 注册 session 模块 |
| `crates/render_view/src/lib.rs` | reset_game_system 调用 resolve_intent + bootstrap；NeedsGameReset 消费 GameIntent |
| `crates/render_view/src/ui/menu.rs` | 主菜单添加"联机"区域 |
| `crates/render_view/src/ui/mod.rs` | 注册 network_panel 系统 |

**不改动：**
- `driver.rs` — 不变（CommandSource enum + trait 不变）
- `network.rs` — 不变
- `transport.rs` — 不变（除新增 ~10 行 GameJoined 通道）
- `relay/` — 不变
- `simulation/` — 不变

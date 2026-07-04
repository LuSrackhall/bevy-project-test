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

/// UI → 逻辑层的纯意图描述。
/// UI 只产生这个，不直接操作 driver 或 network。
/// 符合 UI CLAUDE.md: Widget 是 behavioral building block，产生 semantic event
pub enum GameIntent {
    Single { map_size: MapSize },
    Replay { path: PathBuf },
    Network {
        relay_addr: String,
        player_count: u8,
        // 无 map_size——对局参数由 Relay 协商确认
    },
}
```

```rust
// bevy_adapter/src/session.rs

/// Session 配置，由 resolve_intent() 从 intent 转换而来。
/// 生命周期 = bootstrap 调用后即失效（compile-time irrelevant after bootstrap invocation）。
/// 不得在 runtime tick 路径中访问。
/// 不含 player_id、input_delay、seed——这些属于运行时域或 policy。
pub struct SessionConfig {
    pub mode: SessionMode,
}

pub enum SessionMode {
    Single,
    Replay { path: PathBuf },
    Network { relay_addr: String, player_count: u8 },
}

/// 初始化产物——Initializer 的返回值。
/// 包含初始化完成后的所有依赖对象，供 SessionBootstrap 进行 wiring。
pub struct SessionArtifacts {
    pub source: CommandSource,
    pub transport: Option<TransportResources>,
    // future: replay_reader, metrics, benchmark_state
}

/// Initializer 创建的传输层资源。
/// 由 SessionBootstrap 注册为 Bevy Resources，供 transport poll/flush systems 使用。
pub struct TransportResources {
    pub receiver: NetworkReceiver,
    pub sender: NetworkSender,
    pub handle: NetworkClientHandle,
}
```

### D3: resolve_intent（纯函数，在 render_view 中）

```rust
// render_view/src/session.rs 或 ui/session.rs

pub fn resolve_intent(intent: GameIntent) -> SessionConfig {
    match intent {
        GameIntent::Single { .. } => SessionConfig {
            mode: SessionMode::Single,
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

- **不填充 player_id** — 由 relay 在 handshake 时分配，通过 GameJoined 事件写入
- **不填充 input_delay** — 属于网络 policy，bootstrap 层提供默认值
- **不填充 seed** — relay authoritative
- 100% 纯函数，无 I/O 无副作用无 network access

### D4: SessionInitializer trait（开放-封闭原则）

每种 SessionMode 对应一个 Initializer。每个 Initializer 通过关联类型 `Config` 声明自己需要的配置，编译期保证类型匹配，不需要运行时 match。

```rust
// 每种模式自己的 config 类型
pub struct ReplaySessionConfig { pub path: PathBuf }
pub struct NetworkSessionConfig {
    pub relay_addr: String,
    pub player_count: u8,
}
// SingleInitializer 使用 () 作为 Config——无配置本身也表达"无需参数"

/// SessionInitializer 仅负责产生初始化产物。
/// 不接触 SimulationWorld、Driver、Recorder、cmd_buf。
pub trait SessionInitializer {
    type Config;
    fn initialize(&self, cfg: &Self::Config) -> Result<SessionArtifacts, String>;
}
```

每种 Initializer 只知道自己的 config：

```rust
struct NetworkInitializer;
impl SessionInitializer for NetworkInitializer {
    type Config = NetworkSessionConfig;
    fn initialize(&self, cfg: &NetworkSessionConfig) -> Result<SessionArtifacts, String> {
        let input_delay = 3;
        let (player_id, rx, tx, handle) = connect_and_handshake(&cfg.relay_addr)?;
        let ns = NetworkCommandSource::new(1, player_id, input_delay);
        Ok(SessionArtifacts {
            source: CommandSource::Network(ns),
            transport: Some(TransportResources {
                receiver: rx,
                sender: tx,
                handle,
            }),
        })
    }
}

struct SingleInitializer;
impl SessionInitializer for SingleInitializer {
    type Config = ();
    fn initialize(&self, _cfg: &()) -> Result<SessionArtifacts, String> {
        Ok(SessionArtifacts {
            source: CommandSource::Live(LiveCommandSource),
            transport: None,
        })
    }
}

struct ReplayInitializer;
impl SessionInitializer for ReplayInitializer {
    type Config = ReplaySessionConfig;
    fn initialize(&self, cfg: &ReplaySessionConfig) -> Result<SessionArtifacts, String> {
        let replay = load_replay(&cfg.path)?;
        Ok(SessionArtifacts {
            source: CommandSource::Replay(ReplayCommandSource { replay }),
            transport: None,
        })
    }
}
```

### D4.1 SessionArtifacts Ownership

`SessionArtifacts` 是初始化阶段的一次性所有权对象（one-shot ownership），**必须被恰好消费一次（consumed exactly once）**：

- **Initializer 创建它**
- **`wire()` 必须在一个 pass 内完全分解它（fully deconstruct in one pass）**——将内部资源分别注册到 Driver、Bevy Resources、Recorder，不允许 deferred injection 或 lazy registration
- **`wire()` 完成后，`SessionArtifacts` 不再存在**

**D4.1b：`SessionArtifacts` 是 schema closure invariant（模式闭合不变量）。** 任何新增/修改字段必须经过 initializer + dispatch 通道，不允许在 `wire()` 层扩展 artifact 结构。`wire()` 只消费已存在的字段，不增补、不演化 artifact schema。此约束保证：artifact 的演化路径在 dispatch 和 initializer 中可见，不会出现 wire 层悄悄新增资源注册路径而不被 dispatch 感知。

因此：
- 不应 `Clone`
- 不应 `Rc`/`Arc` 跨线程共享（`TransportResources` 内部的 `Arc` 是 bridge 实现细节，不属于共享 artifact 本身）
- 不应跨 runtime 生命周期保存

重入 bootstrap 会导致：双 driver source overwrite、transport resource leak（多 receiver/sender instance）。D7 的 `SessionActive` 守卫 + D4.1 的 single-consumption semantic 共同防止此问题。

此原则作用于 session-bootstrap-layer 变更引入的抽象，非全局宪法。

SessionBootstrap 分两阶段：dispatch（类型安全）→ wire：

```
Phase 1: initializer.initialize(cfg)  →  SessionArtifacts
Phase 2: SessionBootstrap::wire       → 写 driver.source、insert resources、setup world/recorder
```

```rust
/// 注册：把 SessionMode 映射到具体 Initializer。
/// 这是唯一需要了解所有模式的地方。
fn dispatch(config: &SessionConfig) -> Result<SessionArtifacts, String> {
    match &config.mode {
        SessionMode::Single => {
            SingleInitializer.initialize(&())?
        }
        SessionMode::Replay { path } => {
            ReplayInitializer.initialize(&ReplaySessionConfig { path: path.clone() })?
        }
        SessionMode::Network { relay_addr, player_count } => {
            NetworkInitializer.initialize(&NetworkSessionConfig {
                relay_addr: relay_addr.clone(),
                player_count: *player_count,
            })?
        }
    }
}

/// Wire：把初始化产物接入 Bevy 系统资源。
/// 纯 wiring，不知模式类型。可单测。
fn wire(ctx: &mut InitCtx, artifacts: SessionArtifacts) {
    ctx.driver.source = artifacts.source;
    // 注册传输层资源（如有）
    if let Some(t) = artifacts.transport {
        ctx.insert_resource(t.receiver);
        ctx.insert_resource(t.sender);
        ctx.insert_network_handle(t.handle);
    }
    setup_recorder(ctx);
    init_world(ctx);
}

pub fn bootstrap(config: SessionConfig, ctx: &mut InitCtx) -> Result<(), String> {
    let artifacts = dispatch(&config)?;
    wire(ctx, artifacts);
    Ok(())
}
```

三层各司其职：`dispatch` = registry（可扩展）；`initialize` = I/O（每种模式独立）；`wire` = 系统对接（模式无关）。

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
    pub session_active: &'a mut bool, // bootstrap 重入守卫
}
```

`session_active` 在 bootstrap 入口检查已激活则跳过；出口设 true。防止重入。

### D5.1 wire() invariant

`wire()` 本质是对象图装配阶段（object graph hydration step），不是逻辑函数。只允许执行三类操作：

| 允许的操作 | 示例 |
|-----------|------|
| **assignment** | `driver.source = artifacts.source` |
| **registration** | `insert_resource(transport.receiver)` |
| **deterministic initialization** | `init_world(ctx)`, `setup_recorder(ctx)` |

**禁止：**
- 协议处理（解析或依赖 GameCommand）
- 基于 SessionMode 的条件分支
- I/O（文件读写、网络请求）
- 业务决策（if mode == Network 做不同 setup）

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

**P4/P8 约束：`GameJoined` 是 session-initialization-only barrier，不是 gameplay event。** 它只用于 session creation，**不得代表对已有 session 的重新进入（re-entry）**。reconnect 场景下需通过独立的 `ReconnectResponse` + `ResyncCompleted` 路径处理。`GameJoined` 一旦被用于 bootstrap，其语义即封闭——不可在 runtime 中重新发送或复用。

### D7: SessionActive — 重入守卫

```rust
/// 标记 bootstrap 已完成。防止 scene reload 导致 bootstrap 重入。
pub struct SessionActive;
```

在 `wire()` 末尾通过 `commands.insert_resource(SessionActive)` 插入。`bootstrap()` 入口检查 `InitCtx.session_active`，已激活则跳过。

**P5 约束（Bevy ordering fence）：Bootstrap 完成后需要二次 commit 信号。**

由于 Bevy `Commands` 是 deferred 的（frame boundary commit），`wire()` 中 `driver.source = Network` 立即生效，但 `insert_resource(transport.receiver)` 要下一帧才可见，中间存在一个帧的时间窗口。

解决方案：`wire()` 末尾插入 `SessionBootstrapped` marker：

```rust
pub struct SessionBootstrapped;
// 在 wire() 末尾：commands.insert_resource(SessionBootstrapped);
```

`simulation_driver_system` 在执行 Network tick 前检查 `SessionBootstrapped` 是否存在。不存在则跳过 tick 推进（is_tick_ready 默认 false），等下一帧。

此 marker 与 `SessionActive` 配合：
- `SessionActive` 防止 bootstrap 重入（单次执行守卫）
- `SessionBootstrapped` 防止资源不可见时 tick 提前执行（system ordering fence）

**P6 约束：`driver.source` activation is gated by capability availability (not intent)。** `wire()` 中设 `driver.source = Network` 但不立即激活 `is_tick_ready()`。`NetworkCommandSource` 在收到 `SessionBootstrapped` 之前应返回 `is_tick_ready() = false`（通过内部 `activated` 字段控制）。`SessionBootstrapped` marker 提交后，driver 在下一帧才能真正开始消费 Network source。

**隐含前提（P1）：Bootstrap execution is linear, single-shot, non-overlapping。** 不能并发、不能重入、不能 partial bootstrap（必须 success/fail atomic）。`SessionActive` + `one-shot ownership` 共同保证这一语义。

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
- **[bootstrap 重入]** → `SessionActive` resource 守卫，one-shot 约束
- **[SessionConfig lifecycle]** → bootstrap 调用后即失效，不得在 runtime tick 路径中访问
- **[connect_and_handshake vs transport protocol]** → transport.rs 有约 10 行的适配改动（GameJoined 通道），协议本身不变。设计中已明确这对组改动
- **[handshake 同步阻塞]** → `recv_timeout(5s)` 在 bootstrap 线程同步阻塞。Network mode 下 UI 在连接期间会短暂冻结。**UI 契约：bootstrap 触发前必须显示 "SessionConnecting" 等连接状态指示，否则用户会认为游戏卡死。** 此阻塞在 RTS lockstep 设计中可接受（初始化属于一次性延迟，不影响运行时 tick）
- **[SessionMode 扩展压力]** → 当前 Single / Replay / Network 三种模式通过 `dispatch()` match 管理。未来扩展至 Spectator / Reconnect / Hot join / AI-only 等模式时，`dispatch()` 可能膨胀为 god match。到那时应考虑将 dispatch 从静态 match 演进为 capability composition 模型

---

## 实现路径

| 文件 | 操作 |
|------|------|
| `crates/render_view/src/ui/network_panel.rs` | **新文件** — 联机输入面板 UI（relay 地址 + player_count） |
| `crates/render_view/src/session.rs` | **新文件** — GameIntent enum + resolve_intent() 纯函数 |
| `crates/bevy_adapter/src/transport.rs` | **修改** — 新增 `spawn_network_client_with_game_joined`，约 10 行 |
| `crates/bevy_adapter/src/session.rs` | **新文件** — SessionConfig, SessionMode, SessionArtifacts, SessionBootstrap |
| `crates/bevy_adapter/src/lib.rs` | 注册 session 模块 |
| `crates/render_view/src/lib.rs` | reset_game_system 调用 resolve_intent + bootstrap；NeedsGameReset 消费 GameIntent |
| `crates/render_view/src/ui/menu.rs` | 主菜单添加"联机"区域 |
| `crates/render_view/src/ui/mod.rs` | 注册 network_panel 系统 |

**不改动：**
- `driver.rs` — 不变（CommandSource enum + trait 不变）
- `network.rs` — 不变
- `relay/` — 不变
- `simulation/` — 不变

**微小改动：**
- `transport.rs` — 新增 `spawn_network_client_with_game_joined`（约 10 行），为 bootstrap 提供 GameJoined 通道

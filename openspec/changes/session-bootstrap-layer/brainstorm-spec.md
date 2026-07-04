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

### D1: 分层架构（三层隔离 + 生命周期分界）

```
UI (render_view)
  ↓ GameIntent ← 纯 intent，无 I/O 无状态机
resolve_intent() (pure data transform, 在 render_view 中)
  ↓ SessionConfig ← init-only 数据，不进入 tick path
SessionBootstrap (bevy_adapter 服务函数，由 reset_game_system 调用)
  ↓
Driver → Simulation (不变)
  ↓
transport events (运行时域——GameJoined, TickBatch, Disconnect)
```

**三域生命周期划分：**

| 域 | 范围 | 职责 |
|----|------|------|
| **初始化域** | UI → Intent → Resolver → Bootstrap | 纯数据映射 + 一次性 wiring |
| **运行时域** | Playing → transport → tick loop | 网络事件、driver 循环、simulation |
| **配置域** | SessionConfig | init-only 数据结构，不进入 tick 路径 |

**关键约束：Bootstrap 仅负责初始化，不消费或等待任何运行时网络事件。**
所有握手结果（如 `GameJoined`）、Tick 广播、断线等均属于 Transport Runtime，由 transport 系统处理。Bootstrap 属于初始化域，Transport 属于运行时域，两者生命周期不交叉。

- **GameIntent** = UI → 逻辑层的边界，属于 render_view（符合 UI CLAUDE.md：Widget 产生 semantic event，不持有业务状态）
- **resolve_intent()** = 纯数据转换，属于 render_view。无 I/O、无副作用
- **SessionBootstrap** = bevy_adapter 服务函数，被 `reset_game_system` 调用。唯一的副作用入口（初始化 network client、设置 driver）。由 `SessionActive` resource 守卫，防止重入

### D2: 数据结构

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
/// 生命周期 = init only，不进入 runtime tick 路径。
/// 不含 player_id、input_delay、seed——这些属于运行时域或 policy。
pub struct SessionConfig {
    pub mode: SessionMode,
}

pub enum SessionMode {
    Single,
    Replay { path: PathBuf },
    Network {
        relay_addr: String,
        player_count: u8,
        // 不含 player_id (由 relay 在 handshake 时分配)
        // 不含 input_delay (网络 policy，由 bootstrap 层提供默认值)
        // 不含 seed/map_size (relay authoritative)
    },
}

/// Factory 返回 bundle，非 tuple（防耦合）。
pub struct CommandSourceBundle {
    pub source: CommandSource,
    pub network_handle: Option<NetworkClientHandle>,
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

### D4: SessionBootstrap（bevy_adapter 服务函数）

```rust
// bevy_adapter/src/session.rs

/// 仅做初始化工件，不消费运行时网络事件。
/// 连接握手（GameJoined）属于初始化域——它是"连接是否建立"的确认，不是游戏运行时事件。
/// NetworkCommandSource 从创建起就是完整对象（player_id 是构造参数，不是可变字段）。
pub fn bootstrap(
    config: SessionConfig,
    sim_world: &mut SimulationWorld,
    driver: &mut SimulationDriver,
    recorder: &mut ReplayRecorder,
    cmd_buf: &mut CommandBuffer,
    map_size: MapSize,
    seed: u64,
) -> Result<(), String> {
    match config.mode {
        SessionMode::Single => {
            driver.source = CommandSource::Live(LiveCommandSource);
            Ok(())
        }
        SessionMode::Replay { path } => {
            let replay = load_replay(path)?;
            driver.source = CommandSource::Replay(ReplayCommandSource { replay });
            Ok(())
        }
        SessionMode::Network { relay_addr, player_count } => {
            let input_delay = 3; // 网络 policy 默认值
            let (rx, tx, handle) = spawn_network_client(relay_addr, 1, 0, 1)?;
            // ↑ spawn_network_client 内部阻塞等待 GameJoined 握手完成
            //   返回时连接已建立、player_id 已分配
            //   player_id 通过 GameJoined 消息从 relay 获取，不是占位符
            let ns = NetworkCommandSource::new(1, player_id, input_delay);
            // rx/tx 作为 Bevy Resource 插入，供 transport poll/flush systems 使用
            insert_resource(rx);
            insert_resource(tx);
            insert_resource(handle);
            driver.source = CommandSource::Network(ns);
            Ok(())
        }
    }
}
```

**关于 player_id：** NetworkCommandSource 从创建起就是完整对象。`player_id` 通过阻塞式连接握手（`connect_and_handshake`）从 relay 的 `GameJoined` 响应中获取，不是占位符。这使得 `NetworkCommandSource.player_id` 成为不可变的构造参数，而非运行时可变字段。

### D5: Bootstrap 是唯一允许构造 CommandSource 的入口

> **S1：SessionBootstrap 是唯一允许构造 `CommandSource`、创建 `NetworkClientHandle`、切换 Driver Source 的入口；任何其他系统不得直接修改 Driver 的 `CommandSource`。**

防止 debug 系统、UI 系统或工具系统直接替换 `driver.source`，破坏初始化边界。

### D6: 对局参数的来源

- Network mode 的 **seed** 来自 relay 侧，不是本机随机生成
- Network mode 的 **map_size** 由 relay 协商确认，UI 不提供地图选择（UI 仅输入 relay 地址和玩家数量）
- Single/Replay 模式的 seed 和 map_size 逻辑不变（当前是 UI 选择 + 本机随机 seed）

---

## Risks / Trade-offs

- **[reset_game_system 过载]** → 拆为 intent + resolver + bootstrap，每层单一职责
- **[map_size 双源]** → Network mode 下 relay authoritative，UI 值仅作为 hint
- **[bootstrap 重入]** → `SessionActive` resource 守卫，one-shot 约束
- **[SessionConfig lifecycle]** → 已约束为 init-only，不进入 tick 路径
- **[Bootstrap 等待网络事件]** → 禁止。Bootstrap 仅初始化，不消费运行时事件

---

## 实现路径

| 文件 | 操作 |
|------|------|
| `crates/render_view/src/ui/network_panel.rs` | **新文件** — 联机输入面板 UI（relay 地址 + player_count） |
| `crates/render_view/src/session.rs` | **新文件** — GameIntent enum + resolve_intent() 纯函数 |
| `crates/bevy_adapter/src/session.rs` | **新文件** — SessionConfig, SessionMode, CommandSourceBundle, SessionBootstrap::bootstrap() |
| `crates/bevy_adapter/src/lib.rs` | 注册 session 模块 |
| `crates/render_view/src/lib.rs` | reset_game_system 调用 resolve_intent + bootstrap；NeedsGameReset 消费 GameIntent |
| `crates/render_view/src/ui/menu.rs` | 主菜单添加"联机"区域 |
| `crates/render_view/src/ui/mod.rs` | 注册 network_panel 系统 |

**不改动：**
- `driver.rs` — 不变（CommandSource enum + trait 不变）
- `network.rs` — 不变
- `transport.rs` — 不变（GameJoined 已在 transport 运行时域中处理）
- `relay/` — 不变
- `simulation/` — 不变

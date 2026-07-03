# Session Bootstrap Layer — UI → GameIntent → Driver 入口设计

> 变更名：session-bootstrap-layer
> 关联：宪法 §1.2.7、§2.5.4；v0.4.0 network-command-stream

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

需要新增一条 "Network" 路径，将 UI 输入（relay 地址、player_id）映射为 `CommandSource::Network`，同时保持现有的分层约束。

系统已具备的基础设施：
- relay TCP server（crates/relay/）
- transport.rs（跨线程 bridge + Bevy poll/flush systems）
- NetworkCommandSource + CommandSource::Network 变体

---

## Goals / Non-Goals

**Goals:**
1. 主菜单增加"联机"入口——输入 relay 地址 + 玩家数量
2. 新增 GameIntent → SessionConfig → CommandSource 管道
3. 联机模式下通过 spawn_network_client() 连接 relay、启动 tick loop
4. 联机对局自动录制回放（复用 ReplayRecorder）
5. GameIntentResolver 作为纯数据转换层，不产生副作用
6. SessionBootstrapSystem 作为唯一副作用入口（one-shot，guarded）

**Non-Goals:**
- 不做排位/匹配/房间/大厅系统
- 不做 Observer/观战模式
- 不修改 relay 协议（v0.4.0 已冻结）
- 不修改 Simulation 层
- 不修改 driver 的 tick 循环
- 不做玩家账户/认证

---

## Decisions

### D1: 分层架构（三层隔离）

```
UI (render_view)
  ↓ GameIntent ← 纯 intent，无 I/O 无状态机
GameIntentResolver (pure data transform)
  ↓ SessionConfig ← init-only 数据，不进入 tick path
SessionBootstrapSystem (唯一副作用入口，one-shot guarded)
  ↓
Driver → Simulation (不变)
```

- **GameIntent = UI → 逻辑层的边界。** UI 只产生 intent，不直接修改 driver、不持有 NetworkClientHandle、不访问 relay buffer。
- **Resolver = 纯数据转换。** 无 I/O、无副作用、无 network access。输入 intent，输出 config。
- **Bootstrap = 唯一 side-effect layer。** 初始化 network client、设置 driver、建立 relay 连接的代码全部集中在这里。由 `SessionActive` resource 守卫，防止重入。

### D2: 数据结构

```rust
// crates/bevy_adapter/src/session.rs

/// UI → 逻辑层的纯意图描述。
/// UI 只产生这个，不直接操作 driver 或 network。
pub enum GameIntent {
    Single { map_size: MapSize },
    Replay { path: PathBuf },
    Network {
        relay_addr: String,
        player_count: u8,
        map_size: MapSize,  // UI hint; relay 侧 seed 为 authoritative
    },
}

/// Session 配置，由 Resolver 从 intent 转换而来。
/// 生命周期 = init only，不进入 runtime tick 路径。
pub struct SessionConfig {
    pub mode: SessionMode,
    pub input_delay: u32,
}

pub enum SessionMode {
    Single,
    Replay { path: PathBuf },
    Network {
        relay_addr: String,
        player_id: u8,
        player_count: u8,
    },
}

/// Factory 返回 bundle，非 tuple（防耦合）。
pub struct CommandSourceBundle {
    pub source: CommandSource,
    pub network_handle: Option<NetworkClientHandle>,
}
```

### D3: GameIntentResolver（纯函数）

```rust
pub fn resolve_intent(intent: GameIntent) -> SessionConfig {
    match intent {
        GameIntent::Single { .. } => SessionConfig {
            mode: SessionMode::Single,
            input_delay: 0,
        },
        GameIntent::Replay { path } => SessionConfig {
            mode: SessionMode::Replay { path },
            input_delay: 0,
        },
        GameIntent::Network { relay_addr, player_count, .. } => SessionConfig {
            mode: SessionMode::Network {
                relay_addr,
                player_id: 0, // reserved; relay 实际分配
                player_count,
            },
            input_delay: 3, // default
        },
    }
}
```

不产生 side effect。不访问 network/IO/filesystem。

### D4: SessionBootstrapSystem（one-shot）

Bevy system，在 `OnEnter(GameState::Playing)` 或 detecting pending intent 时触发：

```rust
pub fn bootstrap_session(
    // reads: GameIntent (consumed by resolver)
    // side-effects:
    // - spawn_network_client (if Network)
    // - set driver.source
    // - insert GameMode resource
) {
    if session_active { return; } // one-shot guard
    let config = resolve_intent(take_intent());
    match config.mode {
        SessionMode::Single => {
            driver.source = CommandSource::Live(LiveCommandSource);
        }
        SessionMode::Replay { path } => {
            let replay = load_replay(path);
            driver.source = CommandSource::Replay(ReplayCommandSource { replay });
        }
        SessionMode::Network { relay_addr, .. } => {
            let (rx, tx, handle) = spawn_network_client(relay_addr, ...);
            insert_resource(rx);
            insert_resource(tx);
            insert_resource(handle);
            driver.source = CommandSource::Network(NetworkCommandSource::new(...));
        }
    }
    insert_resource(SessionActive);
}
```

**Guarded by `Resource<SessionActive>`** — 防止 scene reload 或其他 state transition 导致重入。

### D5: map_size / seed / network consistency

- Network mode 的 **seed 来自 relay handshake，不是本地生成。**
- Network mode 的 **map_size 仅作 UI 展示**，不参与 simulation init。Relay 侧是 authoritative（handshake 时下发 map_spec_hash）。
- Single/Replay 模式的 seed 和 map_size 逻辑不变（当前是 UI 选择 + 本地随机 seed）。

### D6: SessionConfig 不进入 tick 路径

- `SessionConfig` 仅用于 `build_command_source()`。
- `driver.clock`, `driver.scheduler`, `source` 等 runtime 数据继续按现有方式管理。
- 没有 system 在 Playing 状态中读取 `SessionConfig`。

---

## Risks / Trade-offs

- **[reset_game_system 过载]** → 拆为 intent + resolver + bootstrap 三层，每层单一职责
- **[map_size 双源]** → Network mode 下 relay authoritative，UI 值仅作为 hint
- **[bootstrap 重入]** → `SessionActive` resource 守卫，one-shot 约束
- **[SessionConfig 生命周期泄露]** → 已约束为 init-only，不进入 tick 路径

---

## 实现路径

| 文件 | 操作 |
|------|------|
| `crates/bevy_adapter/src/session.rs` | **新文件** — GameIntent, SessionConfig, CommandSourceBundle, resolve_intent, bootstrap_session |
| `crates/bevy_adapter/src/lib.rs` | 注册 session 模块 |
| `crates/render_view/src/ui/menu.rs` | 主菜单添加"联机"区域（relay 地址 + player_count 输入） |
| `crates/render_view/src/lib.rs` | NeedsGameReset → GameIntent 消费；reset_game_system 拆分为调用 resolver + bootstrap |
| `crates/render_view/src/ui/mod.rs` | 注册 network_panel 相关系统 |

**不改动：**
- `driver.rs` — 不变
- `network.rs` — 不变
- `transport.rs` — 不变
- `relay/` — 不变
- `simulation/` — 不变

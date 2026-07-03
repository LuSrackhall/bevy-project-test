# RTS CommandStream Protocol v1.0 — 设计文档

> 变更名：network-command-stream
> 关联：宪法 §1.2.7、§2.5.4、§2.5.5、ADR-006

---

## 第 1 节：Context / Goals / Non-Goals

### Context

- Simulation 是确定性状态机（15000 ticks 验证一致）
- 所有**外部输入**导致的状态变化通过 `GameCommand` 驱动。Simulation 内部允许 chain reactions（combat resolution、击杀触发升级等），它们不是 external commands
- `CommandSource` trait 封装 Live / Replay / Network 模式差异
- Replay = command stream log + deterministic re-execution + hash validation
- `cmd_buf` = frame input ingestion buffer（帧输入暂存，非执行层）
- scheduled `CommandBuffer` = simulation 执行来源（由 Driver 注入）
- 宪法强制执行：零感知（§1.2.7）/ pipeline 固化（§2.5.4）/ scheduler 域盲（§2.5.5）

### Goals

1. 支持 2–8 名玩家同一局对战
2. 复用 `CommandSource` trait，不修改 Simulation 层
3. Relay Server 仅做命令收集 + barrier + 广播 + 日志缓存；**不持有 simulation state、不参与 tick 推进决策、不修改命令、不分配排序键**
4. 所有模式可录制：Live / Network / Replay 共用同一 `ReplayRecorder`，录制发生在 **Driver 层**而非 CommandSource 层，通过 `source.should_record()` 而非 `is_live` 类型匹配
5. 断线重连 = replay-based recovery：seed + relay 缓存的完整 command log（不是仅 seed）。客户端从种子重建 world，快速 replay 到当前 tick
6. 输入延迟默认 3 ticks（20Hz → 150ms），可配置

### Non-Goals

1. **不做 Client-Server 权威模型**——Simulation 权限不拆分，Server 不做 truth owner（Phase 1 不进入此方向）
2. **不做 pure P2P lockstep（无 relay）**——NAT 穿透不稳定，reconnect 无中心缓存点
3. **不做 snapshot-based reconnect**——Bevy World 不可序列化，Phase 1 用 replay-based recovery（未来 Phase 可能引入 snapshot 作为性能优化，但不断线重连的默认路径）
4. **不做自定义二进制协议**——Phase 1 用 serde bincode（primary）+ JSON（debug）
5. **不做预测回滚**——Phase 1 lockstep 不需要 speculative execution
6. **不做大厅 / 匹配 / 账户系统**——属于 Phase 2 UX 层

---

## 第 2 节：关键架构决策

### 决策 1：CommandSource trait 扩展

```rust
pub trait CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand>;
    fn total_ticks(&self) -> Option<u32>;

    // 新增（含默认实现）
    fn is_tick_ready(&self, tick: u32) -> bool { true }
    fn should_record(&self) -> bool { true }
}
```

- `is_tick_ready()`：默认 true。仅 NetworkCommandSource 根据远程 CommandBatch 到达状态返回 false。**语义 = "本 tick 所有已知输入是否已收集完成"，不是 "网络是否就绪"。**
- `should_record()`：默认 true，替代现有 `is_live()` 判断。录制独立于传输层类型。

### 决策 2：三层架构边界

```
cmd_buf (local staging)           ← render_view 保持 cmd_buf.push()
NetworkCommandSource (gatherer)    ← 收集本地+远程，通过 is_tick_ready() 做 barrier
Driver (schedule + record)        ← 感知 completeness（is_tick_ready），不感知网络来源
```

NetworkCommandSource 的职责定义为：**收集（gatherer）+ 缓冲 + 完整性判断**。不是 merger（merge 隐式暗示双源融合，会导致未来 replay 分歧）。

Driver 感知 completeness signal，但不感知 network 来源（§2.5.5 域盲）。

### 决策 3：排序规则冻结

当前 `(player_id, sort_tag)` 不变，**禁止引入 `player_seq`**。

Relay 不参与排序决策。同一玩家多条同 `sort_tag` 命令不需要 tiebreaker——它们作用于不同 entity，顺序不影响语义结果。

**安全约束：** Command ordering is deterministic but not semantically prioritized across entities。顺序确定但不表达游戏逻辑优先级。未来如果引入依赖 entity 间顺序的行为（如 AoE + movement combo、queue-based abilities），必须显式在 Action 层面做原子化（合并为一个 Action），而非引入 player_seq。

### 决策 4：Relay Server 职责边界

| 允许 | 禁止 |
|------|------|
| 收集所有玩家 input per tick | 修改命令内容 |
| 确定 tick 完整性（barrier） | 分配排序键 |
| 广播最终 CommandBatch | 执行 simulation |
| 缓存 command log（reconnect 用） | 参与 tick 推进决策 |
| 游戏创建时协调 seed + ruleset_version | 持有 simulation state |
| 基于 (tick, player_id, player_sid) 幂等去重 | 解析或依赖 GameCommand semantic content 做条件分支 |

**核心原则：** Relay is transport barrier, not arbitration layer。

**cmd_buf 的跨模式语义差异（防漂移）：**
- **Live 模式：** cmd_buf 是 `LiveCommandSource.commands_for_tick()` 的读取源（通过 `ctx.bevy_cmds`）。此时它承担 ingestion + execution source 双重角色。
- **Network 模式：** cmd_buf 仅作为上行 staging buffer，`NetworkCommandSource.commands_for_tick()` 只从 relay_buffer 读取（T5），不接触 cmd_buf。
- **约束：** 任何模式都不得在 cmd_buf 中引入"既被 staging 又被 execution consumption 读取"的双重语义。Network 模式下 cmd_buf 的生命周期终点是上行到 relay，不是被 Driver 消费。
- **跨模式 invariant：** `cmd_buf` 在 Network 模式下绝对不得被任何 execution 路径读取——无论当前是否处于 replay、debug 或 tooling 模式。此约束在 Network 模式下无例外。

### 决策 5：方案 A — 严格锁步（单 canonical source）

**最终选型：** Simulation 永远只消费 relay 广播的 CommandBatch。没有 local execution path。

```
render_view
   ↓
cmd_buf (local staging) ──发送给 relay
                               ↓
Relay: collect → barrier → broadcast CommandBatch（含所有玩家，包括发送者自身）
                               ↓
NetworkCommandSource.commands_for_tick() = relay_buffer.get(tick)
                               ↓
Driver → Simulation
```

- 客户端 `commands_for_tick()` 无 merge 逻辑，只从 relay buffer 取
- cmd_buf 在上行后即过期，不再参与 execution
- relay 基于 `(tick, player_id, player_sid)` 做幂等去重（防止重传）
- NoOp 注入是 `(tick, player_id)` 的纯函数，不依赖运行时状态

### 防漂移约束完整表（D1–D15）

| 编号 | 约束 | 类别 |
|------|------|------|
| D1 | is_tick_ready = completeness signal，不是 network readiness | 语义 |
| D2 | should_record 独立于传输层类型 | 录制 |
| D3 | NetworkCommandSource = gatherer（收集+缓冲+完整性判断），不是 merger | 架构 |
| D4 | Driver 感知 completeness，不感知 network | 架构 |
| D5 | 输入延迟偏移只发生在 NetworkCommandSource 内部；NetworkCommandSource 的 tick domain 必须严格等价于 relay tick domain（仅存在 buffer latency，不存在 logical tick transformation） | 时序 |
| D6 | 命令被 finalized batch 消费后清理 buffer，不因迟到就删除 | 生命周期 |
| D7 | NoOp 注入是 (tick, player_id) 的纯函数 | 确定性 |
| D8 | CommandBatch 一旦 finalized 即不可变；relay 不解析 Action semantics | 架构 |
| D9 | Simulation 执行必须被 completeness signal 控制，而非网络时序或到达顺序 | 执行 |
| D10 | 命令注入幂等性：relay 基于 (tick, player_id, player_sid) 去重；客户端不 merge | 注入 |
| D11 | Reconnect 校验 ruleset_version；replay finalized TickCommands，不允许重新排序 | 重连 |
| D12 | 排序语义版本与 GameCommand schema 绑定；schema mismatch 必须显式处理（reject 或 convert），不允许 silent divergence | 版本 |
| D13 | 每个 player_id 在每个 tick 只有一个 canonical command source（= relay CommandBatch）。relay 广播必须包含所有玩家（含发送者自身），self-inclusion 是跨客户端排序一致性的必要条件 | 权威 |
| D14 | Simulation reset(seed) + full TickCommands replay 必须严格等价于 original execution state | 确定性 |
| D15 | 全断开 freeze 时 relay 停止 broadcast CommandBatch → is_tick_ready 返回 false → tick 自然停止 | 容错 |

---

## 第 3 节：Tick Frame 格式与 Tick Barrier 算法

### 3.1 数据结构定义

```rust
/// 语义层（纯命令批次，无网络元数据）
/// 用于 replay 录制、reconnect 日志、seek 恢复
struct TickCommands {
    tick: u32,
    commands: Vec<GameCommand>,
}

/// 传输层帧（网络广播用）
struct BroadcastFrame {
    game_id: u64,
    ruleset_version: u32,
    payload: TickCommands,
    relay_ts_ms: u64,      // debug only：relay 侧 wall clock
}

/// 客户端上行帧
struct PlayerTickFrame {
    magic: u16,             // 协议标识
    game_id: u64,           // 对局标识
    tick: u32,              // 目标 tick（= current + input_delay）
    player_id: u8,
    commands: Vec<GameCommand>,
    player_sid: u64,        // 客户端侧发送序列号（幂等去重用）
}
```

**分层原则（关键）：** replay 存储 = TickCommands 序列，不依赖 BroadcastFrame 的传输元字段。transport（BroadcastFrame）和 log（TickCommands）可以独立演进。

**层归属：** `TickCommands` = simulation artifact（纯仿真产物），`BroadcastFrame` = transport envelope（传输信封）。两者生命周期独立：TickCommands 可被 replay 录制、reconnect 传输、seek 恢复共用；BroadcastFrame 只存在于网络广播阶段。

**重要：** TickCommands 是 relay-finalized deterministic projection，不是 raw input trace。它已经包含 timeout 触发后的 NoOp 注入和排序结果。Replay 录制的是 finalized batch，不是原始输入流。这意味着 replay = 重放 relay 策略输出，而非绕过 relay 直接 replay raw inputs。

### 3.2 Input Delay 数学模型

| 符号 | 定义 |
|------|------|
| `D` | 输入延迟（tick 数）。默认 3 |
| `R` | RTT 的 95 分位值（ms） |
| `T_tick` | 每 tick 的 wall time（50ms @20Hz） |
| `J` | jitter buffer（固定 1 tick） |

**约束：** `D >= R / T_tick + J`，向上取整。默认 D=3 覆盖 `R <= 100ms`。

**数据流：**

```
Real time:   ──N────N+1────N+2────N+3────N+4────N+5──→
                  │      │      │      │
玩家输入          cmd_buf push (target = tick + D)
                  │      │      │      │
Server time:      collect for T              trigger timeout
                  │      │      │      │
Relay broadcast           finalized CommandBatch(T)
                  │      │      │      │
消费 tick                NetworkCommandSource 发出
```

### 3.3 Tick Barrier 算法（Relay 侧）

```
// 每个 relay 维护 tick -> player_inputs 的 buffer
buffer: Map<tick, Map<player_id, Vec<Vec<GameCommand>>>>
ready: Map<tick, Set<player_id>>

fn on_player_frame(frame: PlayerTickFrame):
    buffer[frame.tick][frame.player_id].push(frame.commands)
    ready[frame.tick].insert(frame.player_id)
    try_finalize(frame.tick)

fn try_finalize(tick: u32):
    if ready[tick] == all_players OR is_timed_out(tick):
        let all_cmds = collect_all(tick)

        // 为缺失玩家注入 NoOp
        for pid in all_players - ready[tick]:
            all_cmds.push(GameCommand {
                tick, player_id: pid, action: Action::NoOp
            })

        // 确定性排序：消费端不再重新排序
        all_cmds.sort_by_key(|c| (c.player_id, c.action.sort_tag()))

        // 输出
        let batch = TickCommands { tick, commands: all_cmds }
        broadcast(batch)
        advance_to(tick + 1)

fn is_timed_out(tick: u32) -> bool:
    // timeout 基准是 relay 侧自第一个 frame 到达后的绝对时间
    now_ms() - first_arrival[tick] >= D * T_tick * 1000 + jitter_ms
```

**timeout anchor：** relay wall clock 是权威。`first_arrival[tick]` 记录该 tick 第一个 PlayerTickFrame 到达 relay 的时刻。Timeout 触发后该 tick 立即 finalized，迟到命令放入下一 tick。

**关键：** timeout 不依赖任何客户端的 RTT 测量或心跳。

### 3.4 客户端 Tick 推进模型（方案 A 严格锁步）

```rust
impl NetworkCommandSource {
    // 来自 relay 的 finalized batch
    relay_buffer: HashMap<u32, TickCommands>,
    // 上行 buffer（发送给 relay 前的 staging）
    pending_uplink: Vec<PlayerTickFrame>,

    fn is_tick_ready(&self, tick: u32) -> bool {
        self.relay_buffer.contains_key(&tick)
    }

    fn commands_for_tick(&mut self, tick: u32, _ctx: &DriverContext) -> Vec<GameCommand> {
        // ONLY canonical source = relay finalized batch
        // NO merge with cmd_buf — cmd_buf is purely uplink staging
        self.relay_buffer
            .remove(&tick)
            .map(|b| b.commands)
            .unwrap_or_default()
    }
}
```

- 本地输入通过 `cmd_buf.push()` 进入 → `PlayerTickFrame` 发送给 relay → relay 广播 `BroadcastFrame` 回来 → `commands_for_tick()` 消费 relay batch
- 本地玩家的命令包含在 relay batch 中（relay echo 所有玩家）
- `cmd_buf.retain()` 在 network 模式下仅清理已上行发送的 staging 命令，不影响 execution

### 3.5 Section 3 完整性约束

| 编号 | 内容 |
|------|------|
| T1 | BroadcastFrame payload = TickCommands，两者分层独立演进 |
| T2 | NoOp = (tick, player_id, NoOp.sort_tag=0) 纯函数 |
| T3 | Relay 幂等去重采用 (tick, player_id, player_sid)，不做 semantic dedup |
| T4 | Timeout 基准 = relay wall clock first_arrival，不依赖客户端测量 |
| T5 | Client commands_for_tick() 只 relay_buffer.get(tick)，不访问网络 socket 或 relay 内部状态 |
| T6 | TickCommands 中的排序在 relay finalized 时完成，消费端不重新排序 |

---

## 第 4 节：Reconnect / Error Handling / 工程约束

### 4.1 Reconnect 协议

```
[客户端断线检测: 3 秒无 BroadcastFrame]

ReconnectRequest {
    game_id: u64,
    last_tick_consumed: u32,
}

ReconnectResponse {
    game_id: u64,
    ruleset_version: u32,
    seed: u64,
    map_spec_hash: u64,
    first_tick: u32,                    // == last_tick_consumed + 1
    ticks: Vec<TickCommands>,           // 从 first_tick 到当前已完成的所有 tick
    players: Vec<PlayerState>,          // 存活玩家 ID 列表
}

[客户端流程]
1. 校验 ruleset_version 兼容性
2. init_simulation_world(seed)
3. simulation::map::generate_map(...)
4. 快速 replay ticks（等价于 handle_seek：按 TickCommands 顺序向前驱动，使用同一个 `run_tick_default` 调用，不存在 alternate simulation entry point）
5. 追上当前 tick → 恢复正常 lockstep
```

### 4.2 Error Handling 完整表

| 场景 | 行为 | 关联约束 |
|------|------|---------|
| 单玩家断线 | 其他客户端 tick 继续，该玩家注入 NoOp；relay 保留 command log 供重连 | D7, D15 |
| 全断线 | relay 停止 broadcast CommandBatch → is_tick_ready 返回 false → tick 自然暂停。超时（30s）后 relay 宣布对局结束 | D15 |
| Schema 不兼容 | relay 在握手时返回 INCOMPATIBLE，客户端显示版本不匹配 | D12 |
| PlayerSID 乱序 | relay 拒绝并记录（Phase 2 升级为反作弊） | D10 |
| Observer 模式 | reconnect 后按相同流程重建世界，但不上行 PlayerTickFrame | D14 |
| 上行非幂等重传 | relay 基于 (tick, player_id, player_sid) 去重 | D10 |

### 4.3 工程约束

- **Replay 等价性的形式化定义：** 相同的 `seed + ruleset_version + TickCommands 序列` 执行后必须产生 bitwise-identical simulation state。此定义等价于 D14，tests（`test_set_world_determinism`、`test_driver_live_replay_determinism`）是验证手段，不是定义本身。
- **Replay 记录的格式 = `Vec<TickCommands>` + `ReplayHeader{seed, map_spec_hash, ruleset_version}`**。与 BroadcastFrame 独立。网络模式下 replay 录制在 Driver 层（should_record() = true），录制的是 TickCommands（来自 relay batch）。
- **GameCommand 是无运行时解析的 value object**（已通过 serde derive + field-only struct 保证）。任何需要通过 runtime context 解析才能序列化的字段（如 dynamic ID、system time）不得引入。
- **Replay-based reconnect 等价性**由 existing `test_set_world_determinism` + `test_driver_live_replay_determinism` 覆盖（driver.rs）。

---

## 附录 A：Phase 1 实施范围（最小可行）

协议层涉及的新增/修改代码：

| 组件 | 新增/修改 | 说明 |
|------|----------|------|
| `crates/bevy_adapter/src/driver.rs` | 修改 | CommandSource trait 加 is_tick_ready() + should_record()；ReplayRecorder 条件从 is_live 改为 should_record() |
| `crates/bevy_adapter/src/lib.rs` | 修改 | 导出 NetworkCommandSource |
| `crates/bevy_adapter/src/network.rs` | 新增 | NetworkCommandSource 实现 + relay 通信协议 |
| `crates/bevy_adapter/Cargo.toml` | 修改 | 添加 tokio + bincode 依赖 |
| `crates/simulation/src/command.rs` | 修改 | 无需改 Action，确认序列化 |
| `docs/engineering/command-pipeline-guide.md` | 修改 | 补充 network 模式下的数据流 |

不含：大厅 UI、匹配、账户、observer、spectator。

---

## 附录 B：与宪法的对齐

| 宪法条款 | 对齐方式 |
|---------|---------|
| §1.2.7 Simulation 零感知 | Simulation 完全不知道网络存在。CommandSource trait 隔离 |
| §2.5.4 不变量 a | Simulation 不知道命令来源（NetworkCommandSource 在 Driver 之下） |
| §2.5.4 不变量 b | 所有外部状态变化通过 GameCommand（方案 A 严格执行） |
| §2.5.5 Scheduler 域盲 | Driver 只问 completeness（is_tick_ready），不问网络状态 |
| §2.5.2 排序规则 | (player_id, sort_tag) 不变。relay 在 finalized 时排序 |

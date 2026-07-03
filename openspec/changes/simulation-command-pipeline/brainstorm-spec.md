## Mission

建立 Simulation 的唯一状态修改入口，使 Replay、AI、Scenario、多人联机、断线恢复共享同一条 Command Pipeline，并通过编译期约束和架构守卫保证未来新增功能无法绕过该流水线。**Simulation 永远不知道命令来源，只负责消费 `GameCommand`。**

### Architectural Invariant

```
任何来自 Simulation 外部的状态修改请求，
必须先成为 Scheduled GameCommand，
随后才能影响 Simulation。

Simulation 内部系统允许产生连锁状态更新（如战斗扣血触发升级），
但不得绕过 Scheduled GameCommand 接收新的外部修改。

不存在第二条外部状态修改路径。
```

---

## Context

当前架构存在宪法 §2.5 的落地缺口。宪法要求：

```
所有仿真必须由 GameCommand 驱动（§2.5.1）
实时对局、回放、AI、服务器必须共用同一命令流水线（§2.5.2）
仿真层只消费 CommandBuffer（§2.5.3）
```

但实际存在一条侧门：

```
render_view observer
  → 同时 cmd_buf.push(GameCommand) + 直接修改 SimulationWorld
  → 后者绕过 CommandPipeline → Replay 录不到 → DESYNC
```

这不是单一 bug 修复，而是一次架构固化。Replay 只是受益者之一，真正的目标是：所有外部状态修改请求（Player、Replay、AI、Scenario、Network）必须走同一条路径进入 Simulation，没有第二条路。

### 前期诊断结论

通过 15000 tick driver 层集成测试（3 seeds、Small/Medium 地图、AI+多类命令）已确认：

- Simulation 层（`run_tick_default`）是确定性的 ✅
- Command 注入路径（`commands_for_tick → inject_commands → run_tick_default`）确定性的 ✅
- Seek 路径（forward/backward）确定性的 ✅

现存 DESYNC 仅因 observer 绕过 CommandPipeline 直接修改 Simulation 状态所致。

**Definition — Scheduled GameCommand**：指已绑定目标 Tick、已完成调度、允许被 Simulation 消费的 `GameCommand`。Scheduled 状态之前为 Pending + Scheduler，之后为 Consumed + Discard。

## Goals

### Goal 1: 建立 Simulation 唯一状态修改入口

所有来自 Simulation 外部的状态变更请求，都必须首先表达为 `GameCommand`。Simulation 内部的状态连锁更新不受此约束。

### Goal 2: Simulation 只消费已调度完成（Scheduled）的 GameCommand

Simulation 层的 `run_tick()` 只从已调度队列（当前实现为 `CommandBuffer`）读取命令，不直接消费任何外部输入。对输入来源（键盘、鼠标、网络包、ReplayFile）完全无感。`CommandBuffer` 是 Scheduled Commands 的一种实现，未来可替换为 RingBuffer、Timeline、NetQueue 而不违反此目标。

### Goal 3: 所有外部命令生产者统一抽象为 CommandSource

```rust
trait CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand>;
    fn total_ticks(&self) -> Option<u32>;
}
```

命令生产者包括：`PlayerInput`、`AI`、`NetworkReceiver`、`ReplayCommandSource`、`ScenarioRunner`。AI 不是 Input。Replay 不是 Input。ReplayFile 是数据，`ReplayCommandSource` 才是生产者。Simulation 不关心来源，只消费 `GameCommand`。

`CommandSource` 只负责产生 Scheduled GameCommand，不负责执行命令。Producer → Driver → Simulation 的职责链固定：Producer 生产、Driver 调度、Simulation 执行。

### Goal 4: render_view 等非仿真模块无法获得 Simulation 的可写访问权限（编译期保障）

`NonSendMut<SimulationWorld>` 不再对 `render_view` 开放。通过依赖约束和编译守卫实现。

### Goal 5: 建立 Architecture Guard（CI + 编译约束）

未来新增功能无法绕过命令流水线。包括架构测试（依赖边界检查）和确定性测试（Live→Replay hash 一致）。

### Goal 6: Simulation 与 UI 解耦

Simulation 不知道 Bevy、UI、按钮、Observer、事件的存在。UI 也不知道 Simulation 内部如何运行。中间只有 Command + Query。

此原则已正式写入 ADR-006（Simulation 对下游模块的零感知原则），同时也是 ADR-003（render_view 直接访问 SimulationWorld 的临时许可）的阶段二目标。本次 Change 完成后 ADR-003 标记为 Superseded。

## Non-Goals

- 不改 `hash_world_state` 自身
- 不引入网络传输层（只铺底座，联机本身是后续 Change）
- 不改现有 `Action` 枚举定义（可新增，不重构已有的）
- 不改变 `simulation` 层内部的系统逻辑

## Decisions

### D1: 命名

本次 Change 命名为 `simulation-command-pipeline`。它表达的是架构约束，而非一个实现方式。

### D2: P1 — 消除所有绕过 Command Pipeline 的状态修改

最小改动集：

1. **SpawnType observer**（`hud.rs:288`）：删除 `c.spawn_type = btn.0` 直接修改，只保留 `cmd_buf.push(SetSpawnType)`
2. 全局审查 `render_view` 中所有 `NonSendMut<SimulationWorld>` 取值点，确保无遗漏的绕过路径

验证：`test_driver_live_replay_determinism` 在 P1 前后都通过。

### D3: P2 — 编译期 Guard：SimulationReader + CommandSink

遵循 CQRS 思想，将「读」和「写」拆为两个独立 trait：

```rust
pub trait SimulationReader {
    /// 只读查询 Simulation 世界
    fn query_world<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&simulation::World) -> R;
}

pub trait CommandSink {
    /// 提交一条 GameCommand 进入管道
    fn submit_command(&mut self, cmd: GameCommand);
}
```

- HUD 只显示数据 → 只拿 `SimulationReader`
- 按钮需要下发命令 → 注入 `ResMut<CommandBuffer>` + `cmd_buf.push()`（注意：`SimulationWorld::submit_command()` 推入 simulation 内部 buffer，绕过录制路径。render_view 必须使用 bevy 级 `cmd_buf`。实现细节见 `docs/engineering/command-pipeline-guide.md`）
- Replay Recorder → 只依赖 `CommandSink`
- 以后 AI、Console、Network → 也只依赖 `CommandSink`

两层约束：

**第一层：依赖禁止**

`bevy_adapter::tick::SimulationWorld` 停止对 `render_view` 暴露可变访问。render_view 只能通过 `bevy_adapter` 暴露的这两个 trait 与 Simulation 交互。

**第二层：类型系统约束**

两个 trait 的实现内部持有 `simulation::World`，其 `&self` 签名从类型系统保证外部无法获取 `&mut World`。

**第三层：不可转换约束**

`SimulationReader` 永远不暴露可写 World；`CommandSink` 永远不暴露 World（读或写）。二者之间不存在 `as_any()` → `downcast` → `get_world_mut()` 的转换路径。这是编译期 Guard 的最后一道防线。

**第四层：CommandSink 纯传输接口约束**

`CommandSink::submit_command()` 必须保持纯传输接口（pure transport interface）：不得根据 `GameCommand` 内容做任何分流或逻辑判断（zero semantic branching）。只允许 copy → enqueue → attach metadata（非语义）。

所有语义处理——验证（validation）、鉴权（auth）、优先级（priority）、延迟标记（latency tag）、去重（dedup）——必须发生在 `CommandScheduler` 而非 `CommandSink`。违反此原则将导致 `CommandSink` 退化为 God Entry，并产生 `Sink+Scheduler` 双重验证逻辑分叉风险（replay path pass / live path fail）。

**第五层：SimulationReader API 边界约束**

`SimulationReader` 只能提供**结构查询（structural query）**，不得提供**语义查询 API（semantic query）**。`query_world(|w| ...)` 是结构查询——它暴露的是 World 图结构，而非领域概念。以下 API 属于语义查询，禁止在 `SimulationReader` 上定义：

- ❌ `get_unit_by_id(id) → UnitData`
- ❌ `get_player_units(player) → Vec<UnitId>`
- ❌ `get_enemy_nearby(pos, range) → Vec<UnitId>`

一旦引入语义查询 API，查询能力将与 Simulation 领域模型耦合，使 Reader 成为「第二个 Simulation API surface」。结构查询保持 Reader 在 Simulation 演进时无需跟随修改。

### D4: P3 — CommandSource 统一

```rust
pub trait CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand>;
    /// 有限命令源（Finite Source）返回 Some(total_ticks)，
    /// 如 ReplayFile / Scenario / Benchmark。
    /// 无限命令源（Streaming Source）返回 None，
    /// 如 Live / NetworkReceiver。
    fn total_ticks(&self) -> Option<u32>;
}
```

不包含 `is_replay()` 方法。Driver 不应关心 CommandSource 的具体类型。用 `Option<u32>` 表达有限/无限源的区别，不用 bool 判断来源。

消除 driver 中对 `CommandSource::Replay` 内部字段的全部直接访问（当前 `handle_seek` 和 driver 结束处的 `rs.replay.total_ticks` 访问），替换为 `source.total_ticks()`。

### D5: P4 — 架构测试

**Architecture Tests**：

- render_view 不得获取 `SimulationWorld` 的可写引用
- Driver 对外暴露的唯一操作接口是 `CommandSource`
- 新增 cargo 脚本或 CI step 检查 crate 依赖边界

**Determinism Tests**（已有 + 维护）：

- `test_driver_live_replay_determinism` (15000 tick)
- `test_driver_live_replay_determinism_medium`
- `test_replay_seek_continuation_determinism`
- `test_replay_backward_seek_determinism`

### D6: P5 — Command 生命周期（架构定义）

```
玩家输入 / AI / 网络接收 / ReplayCommandSource / ScenarioRunner
  → Pending   (未入队，等待调度)
  → Command Scheduler (安排到目标 tick，处理延迟补偿 / 预测 / 回滚)
  → Scheduled (在 CommandBuffer 中，绑定具体 tick)
  → Consumed  (被 take_for_tick 取出)
  → Discard   (tick 执行后清理)
  ↓
  写入 ReplayFile (录制时)
  同步到网络 (联机时)
```

当前实现中生命周期是隐式的——`submit_command()` 直接进入 `cmd_buf`。本定义新增 `Command Scheduler` 阶段，为联机场景的延迟补偿和帧预测保留处理位置。当前实现可以跳过 Command Scheduler 直接进入 Scheduled（单机模式），但架构上为它预留了插槽。

Command Scheduler 的未来职责：

允许（纯时间语义）：
- Tick 定位：将 Pending 命令分配到目标 tick
- Tick 重排：网络包到达顺序与 tick 顺序不同时重新排序
- 延迟补偿：调整命令的生效 tick 以对抗网络延迟
- Prediction：预测性提前执行命令，接收 server ack 后修正
- Rollback：接收到 server 回滚通知后撤销已执行命令
- 去重：防止同一条命令被多次调度
- 合并：合并多条同类命令（如同一单位连续 MoveTo）

禁止（领域语义）：
- ❌ 理解 unit / faction / health / position 等 gameplay 概念
- ❌ 基于游戏规则做决策（"这个单位不能移动"）
- ❌ 解释 Entity 类型或状态

**CommandScheduler 约束**：Scheduler 只能理解时间，不得理解游戏。这是整个 pipeline 中最关键的防腐层——一旦 Scheduler 开始依赖 Simulation domain semantics，它就从 temporal layer 退化为 domain layer，侵蚀 Simulation 的纯执行者角色。

每个阶段的责任（Owner 遵循 Truth Ownership 原则，单一所有者）：

| 阶段 | 操作 | 数据位置 | Owner |
|------|------|---------|-------|
| Pending | `CommandSink::submit_command()` → 暂存（注：submit 不等同于接受。Scheduler 可拒绝非法/过期命令） | 调用方 / Command Scheduler 队列 | Command Scheduler |
| Command Scheduler | 网络接收后重新定位 tick（联机模式下） | Command Scheduler 内部 | Command Scheduler |
| Scheduled | `CommandBuffer.push()` | bevy cmd_buf / simulation cmd_buf | Simulation Driver |
| Consumed | `take_for_tick` / `consume_commands_system` | simulation 内部 | Simulation |
| Discard | `retain()` / 下次 tick 清理 | 释放 | 无 |

`CommandSource`（Producer）不拥有 Command 的生命周期。Producer 的职责在 `yield command` 后结束。生命周期（Pending→Scheduled→Consumed→Discard）由 Scheduler→Driver→Simulation 逐级接管。

## Truth Ownership

Simulation 是**游戏状态（Game State）**的唯一 Truth Owner。Command 生命周期遵循单一 Owner 原则，由 Command Scheduler → Simulation Driver → Simulation 顺序接管：

| 阶段 | Owner | 拥有什么 |
|------|-------|---------|
| Pending | Command Scheduler | 未调度的 Command |
| Scheduled | Simulation Driver | 已调度、绑定 tick 的 Command |
| Consumed | Simulation | 游戏状态 (Health, Position, Faction 等) |
| Discard | 无 | — |

外部模块对 Truth 的操作权限仅限于：

| 操作 | 途径 |
|------|------|
| 写（状态修改） | `CommandSink::submit_command()` |
| 读（状态查询） | `SimulationReader::query_world()` |
| 其他所有路径 | ❌ 不允许 |

Mission → Architectural Invariant → Truth Ownership 三者形成闭环：

- **Mission**：建立唯一入口
- **Invariant**：定义什么能改、什么不能改
- **Truth Ownership**：明确谁负责、谁不负责

## Future Work

本次 Change 不实现以下能力，但架构设计（尤其是 Command Scheduler 和 Command 生命周期）为它们保留了插槽：

- Network Scheduler（联机帧同步）
- Rollback（回滚仲裁）
- Prediction（客户端预测）
- Snapshot Sync（断线重连）
- Spectator / Observer（观战模式）
- Dedicated Server（独立服务器）

**Command Normalizer（语义收敛层）**——未来架构 Guarantee，非可选优化。当 AI / 玩家 / 网络 / Replay 等不同 Producer 产生不同粒度的 Command 时（如玩家单发 `MoveUnit` vs AI 批量 `MoveGroup`），Normalizer 承担命令合并、去重、语义压缩、优先级归一职责，防止 Scheduler 退化为 God Layer。位置在 Producer → Sink 之间：

```
Producer
   ↓
Normalizer ← 语义统一点（未来架构 Guarantee）
   ↓
CommandSink
   ↓
CommandScheduler
   ↓
Scheduled → Simulation
```

**Normalizer 约束**：Normalizer is stateless semantic transformation only。它 MUST NOT 做 temporal decisions（tick 分配、冲突仲裁属于 Scheduler）。Normalizer = 语义压缩 / canonicalization；Scheduler = 时间映射 / ordering。二者不得语义重叠。

**约束**：本次设计的任何决策不得封堵上述能力的实现路径。

**Future Work 不豁免 Invariant**：以上所有未来能力（Network、Prediction、Rollback 等）必须继续遵守 Mission 与 Architectural Invariant。不得新增任何绕过 Scheduled GameCommand 的外部状态修改路径。Prediction 不是第二条路——它仍需通过 Scheduler → Scheduled 流程，只是加快了调度速度。

## Risks / Trade-offs

- **[P1 单 tick 延迟]** observer 只推命令不直接改，UI 反馈延迟 1 tick（0.05s） → 用户不可感知。联机模式下这是标配。
- **[P2 编译约束]** render_view 当前多个系统只读读取 SimulationWorld（update_top_bar、selection 等） → 通过 `SimulationReader::query_world()` 仍可读取，需逐个修改参数类型。
- **[P2 API 扩散]** `SimulationReader` 和 `CommandSink` 是 bevy_adapter 对外公开 API，一旦发布后修改成本高。 → 保持最小接口原则，避免一次暴露过多查询能力。用 `query_world(|world| ...)` 而非逐个功能查询方法。
- **[P3 CommandSource 统一]** handle_seek 当前直接访问 ReplaySource.replay 字段 → 在 trait 上加 `total_ticks()` 即可解耦。
- **[P4 架构测试]** 可能因 crate 结构变化变红 → 随架构变化同步更新。

## Migration Plan

```
P1 — 消除绕过 Command Pipeline 的直接修改
 ├ 改动: spawn type observer 删除直接修改，只留命令
 ├ Exit Criteria:
 │   ✓ render_view 中无直接修改 SimulationWorld 的代码
 │   ✓ cargo check
 │   ✓ 所有 replay 确定性测试通过
 └ 进入 P2

P2 — 编译期 Guard（SimulationReader + CommandSink）
 ├ 改动: 定义 trait，替换 render_view 参数类型，切断 NonSendMut 暴露
 ├ Exit Criteria:
 │   ✓ render_view 无 NonSendMut<SimulationWorld>
 │   ✓ cargo check
 │   ✓ Architecture Test 全绿
 │   ✓ Replay 确定性测试通过
 └ 进入 P3

P3 — CommandSource 统一
 ├ 改动: 消除 driver 对 CommandSource 类型的直接判断
 ├ Exit Criteria:
 │   ✓ Driver 测试通过
 │   ✓ handle_seek 不访问 CommandSource 内部字段
 │   ✓ cargo check
 └ 进入 Merge

P4 — 架构测试（并行）
 ├ Architecture Tests + Determinism Tests 持续维护
 └ 贯穿全程

P5 — Command 生命周期架构定义（贯穿全程）
 └ 文档约束
```

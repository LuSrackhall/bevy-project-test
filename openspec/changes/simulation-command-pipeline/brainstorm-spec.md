## Mission

建立 Simulation 的唯一状态修改入口，使 Replay、AI、Scenario、多人联机、断线恢复共享同一条 Command Pipeline，并通过编译期约束和架构守卫保证未来新增功能无法绕过该流水线。**Simulation 永远不知道命令来源，只负责消费 `GameCommand`。**

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

## Goals

### Goal 1: 建立 Simulation 唯一状态修改入口

所有来自 Simulation 外部的状态变更请求，都必须首先表达为 `GameCommand`。Simulation 内部的状态连锁更新不受此约束。

### Goal 2: Simulation 只消费 GameCommand，不直接消费任何外部输入

Simulation 层的 `run_tick()` 只从 `CommandBuffer` 读取命令。对输入来源（键盘、鼠标、网络包、ReplayFile）完全无感。

### Goal 3: 所有外部命令生产者统一抽象为 CommandSource

```rust
trait CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand>;
    fn total_ticks(&self) -> Option<u32>;
}
```

命令生产者包括：`PlayerInput`、`AI`、`NetworkReceiver`、`ReplayCommandSource`、`ScenarioRunner`。AI 不是 Input。Replay 不是 Input。ReplayFile 是数据，`ReplayCommandSource` 才是生产者。Simulation 不关心来源，只消费 `GameCommand`。

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
- 按钮需要下发命令 → 再拿 `CommandSink`
- Replay Recorder → 只依赖 `CommandSink`
- 以后 AI、Console、Network → 也只依赖 `CommandSink`

两层约束：

**第一层：依赖禁止**

`bevy_adapter::tick::SimulationWorld` 停止对 `render_view` 暴露可变访问。render_view 只能通过 `bevy_adapter` 暴露的这两个 trait 与 Simulation 交互。

**第二层：类型系统约束**

两个 trait 的实现内部持有 `simulation::World`，其 `&self` 签名从类型系统保证外部无法获取 `&mut World`。

### D4: P3 — CommandSource 统一

```rust
pub trait CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand>;
    fn total_ticks(&self) -> Option<u32>; // Some for replay/scenario, None for live/network
}
```

不包含 `is_replay()` 方法。Driver 不应关心 CommandSource 的具体类型。Replay 有 `total_ticks`、Live/Network 没有——用 `Option` 表达区别，不用 bool 判断来源。

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
  → Scheduler (安排到目标 tick，处理延迟补偿 / 预测 / 回滚)
  → Scheduled (在 CommandBuffer 中，绑定具体 tick)
  → Consumed  (被 take_for_tick 取出)
  → Discard   (tick 执行后清理)
  ↓
  写入 ReplayFile (录制时)
  同步到网络 (联机时)
```

当前实现中生命周期是隐式的——`submit_command()` 直接进入 `cmd_buf`。本定义新增 `Scheduler` 阶段，为联机场景的延迟补偿和帧预测保留处理位置。当前实现可以跳过 Scheduler 直接进入 Scheduled（单机模式），但架构上为它预留了插槽。

每个阶段的责任：

| 阶段 | 谁操作 | 数据位置 |
|------|--------|---------|
| Pending | `CommandSink::submit_command()` → 暂存 | 调用方 / scheduler 队列 |
| Scheduler | 网络接收后重新定位 tick（联机模式下） | scheduler 内部 |
| Scheduled | `CommandBuffer.push()` | bevy cmd_buf / simulation cmd_buf |
| Consumed | `take_for_tick` / `consume_commands_system` | simulation 内部 |
| Discard | `retain()` / 下次 tick 清理 | 释放 |

## Risks / Trade-offs

- **[P1 单 tick 延迟]** observer 只推命令不直接改，UI 反馈延迟 1 tick（0.05s） → 用户不可感知。联机模式下这是标配。
- **[P2 编译约束]** render_view 当前多个系统只读读取 SimulationWorld（update_top_bar、selection 等） → 通过 `SimulationReader::query_world()` 仍可读取，需逐个修改参数类型。
- **[P2 API 扩散]** `SimulationReader` 和 `CommandSink` 是 bevy_adapter 对外公开 API，一旦发布后修改成本高。 → 保持最小接口原则，避免一次暴露过多查询能力。用 `query_world(|world| ...)` 而非逐个功能查询方法。
- **[P3 CommandSource 统一]** handle_seek 当前直接访问 ReplaySource.replay 字段 → 在 trait 上加 `total_ticks()` 即可解耦。
- **[P4 架构测试]** 可能因 crate 结构变化变红 → 随架构变化同步更新。

## Migration Plan

1. **P1 先行** — 最少阻力，立刻消除已知 DESYNC 源
2. **P3 CommandSource 统一** — driver 层重构，有测试覆盖，可独立验证
3. **P2 编译约束 + P4 架构测试** — 可并行执行：加上约束后贯穿修复 render_view 调用方
4. **P5 生命周期** — 贯穿全程的架构约束

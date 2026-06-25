## Context

当前 bevy_adapter 中有两个独立的 tick 驱动系统：
- `tick_driver_system`：从 Bevy CommandBuffer 取命令，正常速度推进
- `replay_tick_driver_system`：从 ReplayFile 取命令，支持倍速/暂停/seek

两者共享相同的 tick 推进骨架（累积器 → 注入命令 → run_tick），但实现分散。快放/seek 后 AI 行为可能与原始对局不一致，因为两个系统的命令注入和时序处理存在微妙差异。

此外，simulation 层存在 HashMap 非确定性迭代问题（combat/mod.rs、soldier/mod.rs），导致相同输入在不同运行中可能产生不同结果。

项目宪法 2.4 要求：「所有仿真必须由 GameCommand 驱动，使用同一套命令注入与消费流水线」。

## Goals / Non-Goals

**Goals:**
- 统一为单一 tick 驱动系统，命令来源通过 `CommandSource` enum 抽象
- 倍速、暂停、seek 作为驱动参数，只影响每帧 tick 调度密度，不影响每个 tick 内的命令注入顺序和 `run_tick` 的执行语义
- 全场景确定性：任何速度/seek 组合下，相同输入序列产生完全相同的结果
- 为未来 Lockstep 网络、权威服务器留出清晰的扩展路径

**Non-Goals:**
- 不做网络联机（单独 change）
- 不改变 simulation 层的 `run_tick()` 接口
- 不改变 Replay 文件格式
- 不引入 trait 抽象（等 Lockstep 真出现再提取）

## Decisions

### D1: 核心原则

**Driver 决定 How many ticks；Simulation 决定 How one tick executes。**

- Frame 层面：允许变化（1x = 1 tick/frame, 4x = 4 ticks/frame, seek = N ticks/frame）
- Tick 层面：绝对不变（Tick N → inject → run_tick(N) → Tick N+1）

### D2: CommandSource 采用 enum + impl（无 trait）

```rust
pub enum CommandSource {
    Live(LiveCommandSource),
    Replay(ReplayCommandSource),
}
impl CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand> { ... }
    fn is_live(&self) -> bool { ... }
}
```

当前只有 Live/Replay 两种来源，enum + impl 足够。YAGNI 原则：等 Lockstep 出现时再提取 trait。

### D3: SimulationDriver 三层分离

```
SimulationDriver（协调者）
    ├── TickClock         — 时序：accumulator, current_tick（唯一权威）, tick_duration
    ├── SchedulerState    — 调度：pause, speed, seek_target, async_seek
    └── CommandSource     — 命令：Live / Replay
```

- `current_tick` 只在 `TickClock` 中持有
- `SchedulerState` 管理用户交互（暂停/倍速/seek）
- SimulationDriver 通过 `insert_resource(SimulationDriver::new_live())` 注册

### D4: DriverContext 传递运行时依赖

```rust
pub struct DriverContext<'a> {
    pub bevy_cmds: &'a CommandBuffer,
}
```

LiveCommandSource 通过 ctx 访问 Bevy CommandBuffer。DriverContext 为只读上下文。

### D5: 命令消费契约

- 每条命令每 tick 只消费一次
- `commands_for_tick()` 是过滤读取，不消费
- 已消费命令由 `simulation_driver_system` 统一清理，其他系统不得执行此操作

### D6: 录制契约

- 仅在 Live + 录制开启 + 非 seek 时录制
- AI 命令不录制（确定性，从 seed 重新生成）
- async_seek == true 时不录制

### D7: Seek 语义

- 同一 driver 下连续推进多个 tick
- 向后 seek：重新初始化世界后从 0 推进
- 向前 seek：从当前位置推进
- 分帧完成（每帧 500 tick）
- Seek 完成后 `accumulator = 0.0`
- Seek 期间 `is_seeking = true`，render_view 冻结渲染系统

### D8: 统一系统调度 + GameMode 门控

```rust
// 统一驱动
simulation_driver_system.before(sync_entities_system)
    .run_if(GameActive(true))

// 视觉系统（回放时也运行）
visual_systems.run_if(GameActive(true) && !Paused(true) && !replay_seeking)

// 输入系统（仅 Live 模式）
input_systems.run_if(GameActive(true) && !Paused(true) && !replay_seeking && !GameMode::Replay)
```

- `GameMode` 枚举（Live/Replay）作为轻量门控
- 输入系统在回放时不运行，防止干扰仿真
- 视觉系统在两种模式都运行

### D9: TickClock 双份同步

`SimulationDriver.clock` 是权威时钟，`TickClock` 作为独立 Resource 注册（presentation 层兼容）。`simulation_driver_system` 每帧同步 current_tick 和 accumulator。

### D10: HashMap + 排序遍历

simulation 层位置查询使用 `HashMap`（O(1) 查找），遍历前对 keys 排序保证确定性：
```rust
let mut sorted_ids: Vec<UnitId> = positions.keys().copied().collect();
sorted_ids.sort();
```

`enemy_positions` 保留 `BTreeMap`（最近敌人 tie-break 专用）。

### D11: pending.events 清理

`simulation_driver_system` 在帧首调用 `pending.events.clear()`，与旧 tick_driver_system 行为一致。

### D12: world_fingerprint 工具

保留 `world_fingerprint` 函数（`#[allow(dead_code)]`），用于确定性调试。需要时临时加日志即可定位回放分叉点。

## Risks / Trade-offs

- [tick_duration: f32] → 当前 20Hz 下精确，未来联机时审视
- [HashMap + 排序遍历] → 10 万+ 实体时排序开销 O(n log n)，可接受
- [TickClock 双份] → 需保持同步，已每帧更新
- [GameMode 与 SimulationDriver 分离] → 轻量门控 vs 统一状态，当前方案平衡了关注点分离

## 关键不变量

| # | 不变量 | 验证方式 |
|---|--------|---------|
| I1 | 相同 seed + 相同命令序列 → 相同世界状态 | 黄金测试（93 个） |
| I2 | 每个 Tick 都严格执行同一流水线：commands_for_tick → inject_commands → run_tick | 代码审查 |
| I3 | Live 命令每 tick 只消费一次 | 代码审查 |
| I4 | seek 完成后 accumulator 为 0 | 测试 |
| I5 | seek 期间不录制 | 代码审查 |
| I6 | TickClock.current_tick 是唯一权威 tick 值 | 代码审查 |
| I7 | SimulationDriver 不得直接修改 SimulationWorld | 代码审查 |
| I8 | simulation 层 HashMap 遍历必须排序保证确定性 | 代码审查 + 黄金测试 |

## 验证策略

**已有测试**：93 个 simulation 层测试（含黄金确定性、seek 确定性、e2e replay 测试）

**新增测试**：
- Driver 层确定性测试（4 个）
- Replay e2e 测试（录制→序列化→反序列化→回放→状态一致）

**CI 门控**：
```bash
cargo test -p simulation      # 93+ 测试
cargo test -p bevy_adapter    # Driver 测试
cargo test -p simulation --lib replay  # Replay 回归
cargo check                   # 全项目编译
```

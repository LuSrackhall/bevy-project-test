## Context

当前 bevy_adapter 中有两个独立的 tick 驱动系统：
- `tick_driver_system`：从 Bevy CommandBuffer 取命令，正常速度推进
- `replay_tick_driver_system`：从 ReplayFile 取命令，支持倍速/暂停/seek

两者共享相同的 tick 推进骨架（累积器 → 注入命令 → run_tick），但实现分散。快放/seek 后 AI 行为可能与原始对局不一致，因为两个系统的命令注入和时序处理存在微妙差异。

项目宪法 2.4 明确要求：「所有仿真必须由 GameCommand 驱动，使用同一套命令注入与消费流水线」。当前两个系统违反了这一原则。

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

- Frame（渲染帧）层面：允许变化（1x = 1 tick/frame, 4x = 4 ticks/frame, seek = N ticks/frame）
- Tick（仿真）层面：绝对不变（Tick N → inject commands → run_tick(N) → Tick N+1）

### D2: CommandSource 采用 enum + impl（无 trait）

```rust
pub enum CommandSource {
    Live(LiveCommandSource),
    Replay(ReplayCommandSource),
}

impl CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand> {
        match self {
            Self::Live(s) => s.commands_for_tick(tick, ctx),
            Self::Replay(s) => s.commands_for_tick(tick, ctx),
        }
    }
}
```

**理由**：当前只有 Live/Replay 两种来源，enum + impl 足够。YAGNI 原则：等 Lockstep 出现时再提取 trait，外部调用点基本不用改。

**归属**：bevy_adapter 层（Driver 层），不属于 simulation。simulation 只认 `CommandBuffer`。

### D3: SimulationDriver 三层分离

```
SimulationDriver（协调者）
    ├── TickClock         — 时序：accumulator, current_tick（唯一权威）, tick_duration
    ├── SchedulerState    — 调度：pause, speed, seek_target, async_seek
    └── CommandSource     — 命令：Live / Replay
```

- `current_tick` 只在 `TickClock` 中持有，是唯一权威值
- `ReplayCommandSource` 不持有 tick 状态
- `SchedulerState` 管理用户交互（暂停/倍速/seek），与命令来源解耦

### D4: DriverContext 传递运行时依赖

```rust
pub struct DriverContext<'a> {
    pub bevy_cmds: &'a CommandBuffer,  // Bevy 侧命令缓冲（Live 模式需要）
}
```

`LiveCommandSource` 通过 `ctx.bevy_cmds` 访问 Bevy CommandBuffer，不直接持有引用。`DriverContext` 定义为只读上下文。

### D5: 命令消费契约

- 每条命令每 tick 只消费一次
- `commands_for_tick()` 是过滤读取，不消费 Bevy CommandBuffer
- `run_tick()` 内部的 `consume_commands_system` 执行一次性消费
- 消费后清除 Bevy CommandBuffer 中已处理的命令：`retain(|c| c.tick > current_tick)`
- Bevy CommandBuffer 允许提前存在未来 tick 的命令，`retain` 不会误清

### D6: 录制契约

- 仅在 Live + 录制开启 + 非 seek 时录制外部命令
- AI 命令（run_tick 内部产生）不录制（确定性，从 seed 重新生成）
- async_seek == true 时不录制（seek 是仿真重建，不是游戏过程）

### D7: Seek 语义

- Seek 不是"跳过若干 tick 的帧调度"，而是"在同一 driver 下连续推进多个 tick"
- 向后 seek：重新初始化世界后从 0 推进到目标
- 向前 seek：从当前位置推进到目标
- 分帧完成（每帧 500 tick），`async_seek` 在到达目标前保持 true
- Seek 完成后 `accumulator = 0.0`，防止残留累积器导致 tick 漂移
- Seek 期间 `is_seeking = true`，render_view 据此冻结渲染系统

### D8: 统一系统调度

```rust
// 替换 tick_driver_system + replay_tick_driver_system
.add_systems(Update, (
    simulation_driver_system.before(sync_entities_system),
    sync_entities_system,
).run_if(resource_exists_and_equals(GameActive(true))))
```

- 不再需要 GameMode 的 run_if 条件
- GameActive 是唯一的外部门控
- 系统顺序显式声明（.before()），不依赖 tuple 顺序

### D9: SimulationDriver 辅助方法

```rust
impl SimulationDriver {
    fn is_replay(&self) -> bool {
        matches!(self.source, CommandSource::Replay(_))
    }
}
```

render_view 通过此方法检查模式，不直接 `matches!` 枚举。

### D10: ReplayStatus 边界

```rust
pub struct ReplayStatus {
    pub is_replay: bool,     // 回放元数据
    pub total_ticks: u32,    // 回放元数据（进度条用）
    pub is_seeking: bool,    // 运行态（render_view 据此冻结渲染）
}
```

单向数据流：bevy_adapter 写入 → render_view 只读。UI 只提交控制意图（修改 scheduler），驱动层负责落地。

## Risks / Trade-offs

- [tick_duration: f32] → 当前 20Hz 下 0.05f32 精确，不改。未来联机时审视是否需要 f64 或整数化。
- [SimulationDriver 不直接修改 World] → I7 不变量约束。唯一合法路径：commands → inject → run_tick。
- [bevy_adapter 测试复杂度] → Driver 层测试需要 Bevy App。折中：先写纯逻辑的 accumulator 推进测试。

## 关键不变量

| # | 不变量 | 验证方式 |
|---|--------|---------|
| I1 | 相同 seed + 相同命令序列 → 相同世界状态 | 黄金测试（已有） |
| I2 | 每个 Tick 都严格执行同一流水线：commands_for_tick → inject_commands → run_tick，不存在绕过该流水线的执行路径 | 代码审查（整个 crate 中只有 simulation_driver_system 调用 run_tick） |
| I3 | Live 命令每 tick 只消费一次 | 测试 |
| I4 | seek 完成后 accumulator 为 0 | 测试 |
| I5 | seek 期间不录制 | 代码审查 |
| I6 | TickClock.current_tick 是唯一权威 tick 值 | 代码审查（无其他 tick 持有者） |
| I7 | SimulationDriver 不得直接修改 SimulationWorld，唯一合法路径：commands → inject → run_tick | 代码审查 |

## 验证策略

**已有测试（保留）**：93 个 simulation 层测试（含黄金确定性、seek 确定性）

**新增测试**：
- `test_speed_determinism`：Driver 层验证不同调度密度下结果一致
- `test_seek_determinism`：seek 后继续播放与连续播放结果一致
- `test_command_single_consumption`：Live 命令每 tick 只消费一次
- `test_seek_clears_accumulator`：seek 后 accumulator 为 0

**CI 门控**：
```bash
cargo test -p simulation      # 93+ 测试
cargo test -p bevy_adapter    # Driver 测试
cargo test replay             # Replay 回归（独立门禁）
cargo check                   # 全项目编译
```

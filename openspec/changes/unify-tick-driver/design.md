## Context

当前 bevy_adapter 中 `tick_driver_system` 和 `replay_tick_driver_system` 是两个独立系统，各自实现 tick 推进逻辑。本设计将它们统一为 `SimulationDriver` + `CommandSource` 架构。

## Goals / Non-Goals

详见 brainstorm-spec.md。

## Decisions

### D1: 核心原则

**Driver 决定 How many ticks；Simulation 决定 How one tick executes。**

- Frame 层面：允许变化（1x = 1 tick/frame, 4x = 4 ticks/frame, seek = N ticks/frame）
- Tick 层面：绝对不变（Tick N → inject → run_tick(N) → Tick N+1）

### D2: CommandSource enum + impl（无 trait）

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
    fn is_live(&self) -> bool { matches!(self, Self::Live(_)) }
}
```

当前只有 Live/Replay 两种来源，enum + impl 足够。等 Lockstep 出现时再提取 trait。

### D3: SimulationDriver 三层分离

```rust
#[derive(Resource)]
pub struct SimulationDriver {
    pub clock: TickClock,
    pub scheduler: SchedulerState,
    pub source: CommandSource,
}

pub struct TickClock {
    pub current_tick: u32,       // 唯一权威值
    pub tick_duration: f32,      // 0.05 = 20Hz
    pub accumulator: f32,
}

pub struct SchedulerState {
    pub is_paused: bool,
    pub speed_multiplier: u32,
    pub seek_target: Option<u32>,
    pub async_seek: bool,
}

pub struct LiveCommandSource;  // 无状态，通过 ctx 取命令
pub struct ReplayCommandSource { pub replay: ReplayFile }
```

### D4: DriverContext

```rust
pub struct DriverContext<'a> {
    pub bevy_cmds: &'a CommandBuffer,
}
```

LiveCommandSource 通过 ctx 访问 Bevy CommandBuffer。DriverContext 为只读上下文。

### D5: 命令消费契约

- 每条命令每 tick 只消费一次
- `commands_for_tick()` 是过滤读取，不消费
- `run_tick()` 内部执行一次性消费
- 已消费命令由 `simulation_driver_system` 统一清理，其他系统不得执行此操作

### D6: 录制契约

- 仅在 Live + 录制开启 + 非 seek 时录制
- AI 命令不录制
- async_seek == true 时不录制

### D7: Seek 语义

- 同一 driver 下连续推进多个 tick
- 向后 seek：重新初始化世界后从 0 推进
- 向前 seek：从当前位置推进
- 分帧完成（每帧 500 tick）
- Seek 完成后 accumulator = 0.0
- Seek 期间 is_seeking = true

### D8: 统一系统调度

```rust
.add_systems(Update, (
    simulation_driver_system.before(sync_entities_system),
    sync_entities_system,
).run_if(resource_exists_and_equals(GameActive(true))))
```

GameActive 是唯一外部门控。系统顺序用 .before() 显式声明。

### D9: SimulationDriver::is_replay()

render_view 通过此方法检查模式，不直接 matches 枚举。

### D10: ReplayStatus 边界

```rust
pub struct ReplayStatus {
    pub is_replay: bool,     // 展示态缓存（派生自 source，非权威状态）
    pub total_ticks: u32,
    pub is_seeking: bool,    // 运行态
}
```

单向数据流：bevy_adapter 写入 → render_view 只读。UI 只提交控制意图，驱动层负责落地。

## Risks / Trade-offs

- [tick_duration: f32] → 当前 20Hz 下精确，未来联机时审视
- [SimulationDriver 不直接修改 World] → I7 不变量约束
- [bevy_adapter 测试复杂度] → 先写纯逻辑的 accumulator 推进测试

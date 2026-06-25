## Context

当前 bevy_adapter 中 `tick_driver_system` 和 `replay_tick_driver_system` 是两个独立系统，各自实现 tick 推进逻辑。本设计将它们统一为 `SimulationDriver` + `CommandSource` 架构，并解决 simulation 层的 HashMap 非确定性问题。

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
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand> { ... }
    fn is_live(&self) -> bool { matches!(self, Self::Live(_)) }
}
```

当前只有 Live/Replay 两种来源，enum + impl 足够。等 Lockstep 出现时再提取 trait。

### D3: SimulationDriver 三层分离

```rust
pub struct SimulationDriver {
    pub clock: TickClock,
    pub scheduler: SchedulerState,
    pub source: CommandSource,
}
```

- `current_tick` 只在 `TickClock` 中持有，是唯一权威值
- `SchedulerState` 管理用户交互（暂停/倍速/seek），与命令来源解耦

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
- AI 命令不录制（确定性，从 seed 重新生成）
- async_seek == true 时不录制

### D7: Seek 语义

- 同一 driver 下连续推进多个 tick
- 向后 seek：重新初始化世界后从 0 推进
- 向前 seek：从当前位置推进
- 分帧完成（每帧 500 tick）
- Seek 完成后 accumulator = 0.0
- Seek 期间 is_seeking = true，render_view 冻结渲染系统

### D8: 统一系统调度 + GameMode 门控

```rust
// 统一驱动：GameActive 门控
simulation_driver_system.before(sync_entities_system)
    .run_if(GameActive(true))

// 输入系统：GameActive + GameMode::Live 双重门控
input_systems.run_if(GameActive(true) && !GameMode::Replay)
```

- `GameMode` 枚举（Live/Replay）作为轻量门控资源
- 输入系统（command_issue 等）仅在 Live 模式运行
- 视觉系统在两种模式都运行

### D9: TickClock 双份同步

`SimulationDriver.clock` 是权威时钟，但 `TickClock` 仍作为独立 Resource 注册（presentation 层兼容）。`simulation_driver_system` 每帧同步：
```rust
tick_clock.current_tick = driver.clock.current_tick;
tick_clock.accumulator = driver.clock.accumulator;
```

### D10: HashMap + 排序遍历

simulation 层的位置查询使用 `HashMap`（O(1) 查找），遍历前对 keys 排序保证确定性：
```rust
let mut sorted_ids: Vec<UnitId> = positions.keys().copied().collect();
sorted_ids.sort();
for id in sorted_ids { ... }
```

`enemy_positions` 保留 `BTreeMap`（最近敌人扫描需要 tie-break 确定性）。

### D11: pending.events 清理

`simulation_driver_system` 在帧首调用 `pending.events.clear()`，与旧 `tick_driver_system` 行为一致。

### D12: SimulationDriver 注册

`SimulationDriver` 通过 `insert_resource(SimulationDriver::new_live())` 注册（不实现 Default trait）。

## Risks / Trade-offs

- [tick_duration: f32] → 当前 20Hz 下精确，未来联机时审视
- [HashMap + 排序遍历] → 10 万+ 实体时排序开销 O(n log n)，可接受
- [TickClock 双份] → 需保持同步，已每帧更新
- [GameMode 与 SimulationDriver 分离] → 轻量门控 vs 统一状态，当前方案平衡了关注点分离

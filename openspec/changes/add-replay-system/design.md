## Context

本设计基于 brainstorm-spec.md 中经 5 路评审确认的方案。项目为 Bevy 0.18 RTS 游戏，严格分层架构 `simulation ← bevy_adapter ← presentation ← render_view`。仿真层使用 Fixed(i64) 定点数和 CommandBuffer 驱动，但存在 12-15 处 f32 泄漏。Replay 系统是验证确定性并通向 P2P 联机的第一步。

## Goals / Non-Goals

**Goals:**
- 详见 brainstorm-spec.md 的 Goals 部分
- 本文档聚焦实现层面的架构决策和模块边界

**Non-Goals:**
- 详见 brainstorm-spec.md 的 Non-Goals 部分

## Decisions

### D1: f32→万分比整数的转换策略

**配置层改造**：content/*.ron 中所有 `f32` 概率/比率字段改为 `u32` 万分比。
- `pierce_chance: 0.3` → `pierce_chance: 3000`（表示 30.00%）
- `heal_ratio: 0.05` → `heal_ratio: 500`（表示 5.00%）
- `stack_mult: 0.8` → `stack_mult: 8000`（表示 80.00%）

**运行时计算**：所有概率比较改为整数域。
```
旧: rng.gen_probability() < 0.3
新: rng.gen_probability_permille() < 3000   // gen 返回 0..10000
```

**比例乘法**：整数乘法 + 万分比除法。
```
旧: hp as f32 * ratio
新: (hp as u64 * ratio as u64) / 10000
```

**powi 替换**：减速倍率的 `powi(n)` 改为循环定点数乘法。
```
旧: base.powi(stacks - 1)
新: let mut result = 10000u32; for _ in 0..stacks-1 { result = result * base / 10000; }
```

**理由**：万分比提供 0.01% 精度，覆盖所有游戏配置需求。Fixed 的 8-bit 精度（1/256 ≈ 0.39%）对概率比较不足。

### D2: gen_probability 改造

`DeterministicRng` 新增方法：
```rust
/// 返回 0..10000 的确定性概率值（万分比）
pub fn gen_probability_permyriad(&mut self) -> u32 {
    (self.0.next_u64() % 10001) as u32
}
```

保留原 `gen_probability() -> f32` 仅供 presentation 层使用（如视觉效果随机化），但在 simulation 层禁用。

### D3: HashMap→BTreeMap 改造

`combat/mod.rs` 中 `enemy_positions: HashMap<UnitId, FixedVec2>` 改为 `BTreeMap<UnitId, FixedVec2>`。当两个敌人距离完全相同时，BTreeMap 按 UnitId 排序保证迭代顺序确定。

### D4: Replay 文件格式

使用 RON 格式（开发阶段），文件头含版本信息：
```
ReplayFile {
    format_version: 1,
    seed: 12345,
    map_size: Medium,
    total_ticks: 12000,
    commands_per_tick: { 5: [...], 12: [...], ... },
}
```

bincode 格式在 Phase 3 引入，通过文件头魔数区分格式。

### D5: 录制拦截点

在 bevy_adapter 的 `tick_driver_system` 中，命令从 Bevy 侧 CommandBuffer 提取后、注入 simulation 前，复制一份到 ReplayRecorder。

关键区分：
- **外部命令**（玩家通过 CommandBuffer 注入的）→ 录制
- **AI 命令**（在 run_tick 内部 ai_decide 产生的）→ 不录制（确定性，从 seed 重新生成）

### D6: Replay 播放控制架构

```rust
// bevy_adapter 新增资源
#[derive(Resource)]
pub enum GameMode {
    Live,
    Replay(ReplayController),
}

pub struct ReplayController {
    pub replay: ReplayFile,
    pub current_tick: u32,
    pub target_tick: Option<u32>,   // None = 播放到末尾
    pub speed: ReplaySpeed,
    pub is_paused: bool,
    pub is_seeking: bool,
}

pub enum ReplaySpeed {
    Normal,     // 1x，每帧 1 tick
    Fast2x,     // 2x
    Fast4x,     // 4x
    SeekTo(u32), // 快放到目标 tick
}
```

`replay_tick_driver_system` 与 `tick_driver_system` 通过 `run_if` 条件互斥运行。

### D7: render_view 进度条数据流

```
bevy_adapter: GameMode → ReplayStatus { is_replay: bool, total_ticks: u32 }
render_view: 读取 ReplayStatus + TickClock.current_tick → 渲染进度条
render_view: 用户拖拽进度条 → 设置 GameMode.Replay.target_tick
```

render_view 不 import 任何 simulation 类型，只通过 bevy_adapter 的轻量资源交互。

## Risks / Trade-offs

- [f32→整数精度损失] → 万分比 0.01% 精度足够。极端情况（如 0.001% 概率）在当前游戏中不存在。
- [seek 性能] → Phase 2 接受 10 分钟对局 ~6 秒 seek。Phase 3 引入 silent tick + 快照优化。
- [配置迁移工作量] → 约 20 个字段需要从 f32 改为 u32，同步更新所有使用处。模式统一，机械性工作。
- [SmallRng 跨版本] → 锁定 Cargo.lock。未来可替换为自实现的确定性 RNG（如 xoshiro256++）。

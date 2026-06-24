## Context

本设计基于 brainstorm-spec.md 中经 5 路评审确认的方案。项目为 Bevy 0.18→0.19 RTS 游戏，严格分层架构 `simulation ← bevy_adapter ← presentation ← render_view`。仿真层使用 Fixed(i64) 定点数和 CommandBuffer 驱动，但存在 12-15 处 f32 泄漏。Replay 系统是验证确定性并通向 P2P 联机的第一步。

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
新: rng.gen_probability_permyriad() < 3000   // gen 返回 0..10000
```

**比例乘法**：整数乘法 + 万分比除法。
```
旧: hp as f32 * ratio
新: (hp as u64 * ratio as u64) / 10000
```

**powi 替换**：减速倍率的 `powi(n)` 改为循环万分比乘法。
```
旧: base.powi(stacks - 1)
新: let mut mult_pm = 10000u64; for _ in 0..(stacks-1) { mult_pm = mult_pm * base as u64 / 10000; }
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

保留原 `gen_probability() -> f32`（标记 deprecated）仅供 presentation 层使用。

### D3: HashMap→BTreeMap 改造

`combat/mod.rs` 中 `enemy_positions: HashMap<UnitId, FixedVec2>` 改为 `BTreeMap<UnitId, FixedVec2>`。

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

bincode 格式在未来引入，通过文件头魔数区分格式。

### D5: 录制拦截点

在 bevy_adapter 的 `tick_driver_system` 中，命令从 Bevy 侧 CommandBuffer 提取后、注入 simulation 前，复制一份到 ReplayRecorder。AI 命令不录制（确定性，从 seed 重新生成）。

### D6: Replay 播放控制架构（实际实现）

```rust
// bevy_adapter: GameMode 为独立枚举，ReplayController 为独立资源
#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameMode { #[default] Live, Replay }

#[derive(Resource)]
pub struct ReplayController {
    pub replay: ReplayFile,
    pub current_tick: u32,
    pub is_paused: bool,
    pub speed_multiplier: u32,    // 1, 2, 4, 8, 16
    pub seek_target: Option<u32>,
    pub async_seek: bool,         // 多帧异步 seek 标志
}

#[derive(Resource, Default)]
pub struct ReplayStatus {
    pub is_replay: bool,
    pub total_ticks: u32,
    pub is_seeking: bool,         // seek 期间冻结渲染
}
```

- `replay_tick_driver_system` 驱动正常回放（按累积时间推进 tick）
- `replay_seek_system`（在 render_view 中）处理异步 seek（每帧 500 tick，backward 需重置世界）
- 两个系统通过 `async_seek` 标志协调，tick_driver 在 `async_seek=true` 时跳过
- seek 期间 `is_seeking=true` 冻结渲染系统，全力处理 tick

### D7: UI 控件设计（实际实现）

回放控制栏包含：
- `<< 10s` — 快退 10 秒（异步 seek，重新初始化世界 + 快放）
- `||` / `>` — 暂停/播放
- `10s >>` — 快进 10 秒（异步 seek）
- `1x 2x 4x 8x 16x` — 播放速度控制
- 进度条 — 纯视觉显示（不支持拖拽 seek）
- 时间显示 — M:SS 格式

不支持拖拽 seek（仿真回放不是视频，跳转需重放，体验不可接受）。

### D8: render_view 数据流

```
bevy_adapter: ReplayStatus { is_replay, total_ticks, is_seeking }
render_view: 读取 ReplayStatus + ReplayController.current_tick → 渲染 UI
render_view: 不 import simulation 类型，只通过 bevy_adapter 资源交互
```

## Risks / Trade-offs

- [f32→整数精度损失] → 万分比 0.01% 精度足够。
- [seek 性能] → 每帧 500 tick，10 分钟对局回退到开头约 24 帧 ≈ 1.2 秒。
- [backward seek 需要重置世界] → 无法倒放，只能从头快放。快退 10 秒几乎瞬间。
- [进度条不支持拖拽] → 仿真回放不适合拖拽 seek，用快退/快进按钮替代。
- [SmallRng 跨版本] → 锁定 Cargo.lock。

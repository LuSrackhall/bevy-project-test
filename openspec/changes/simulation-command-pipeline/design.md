## Context

参考 brainstorm-spec.md 获取完整的高层设计、决策树和架构不变量体系。本文件聚焦实现层面的关键接口和迁移策略。

当前架构违反宪法 §2.5：render_view 的 observer 可以直接修改 SimulationWorld（如 `c.spawn_type = btn.0`），绕过 CommandPipeline。这导致 Replay 无法录制该修改，产生 DESYNC。

## Goals / Non-Goals

**Goals:**
- P1: 消除所有绕过 CommandPipeline 的直接 SimulationWorld 修改
- P2: 引入 `SimulationReader` + `CommandSink` 替代 render_view 对 SimulationWorld 的可写访问
- P3: CommandSource 统一，消除 driver 对来源类型的直接判断
- 宪法 §2.5.4 (a-d) + §2.5.5 落地

**Non-Goals:**
- 不改变 simulation 层内部逻辑（consume_commands_system 已正确）
- 不引入 Network / AI / Scenario 的实际 CommandSource 实现
- 不实现 Command Scheduler（留为 P5 架构位置）
- 不实现 Command Normalizer

## Decisions

### D1: SimulationReader（只读）+ CommandSink（命令提交）

```rust
// 在 bevy_adapter 中定义
pub trait SimulationReader {
    fn query_world<F, R>(&self, f: F) -> R
    where F: FnOnce(&simulation::World) -> R;
}

pub trait CommandSink {
    fn submit_command(&mut self, cmd: GameCommand);
}
```

Reader 和 Sink 在 render_view 中通过正常的 Bevy 系统参数注入。实现层内部持有 `simulation::World`，但 `&self` 签名保证 `query_world` 内无法获取 `&mut World`。

**约束链（5 层）** 见 brainstorm-spec.md D3。

### D2: P1 直接消除

在 `hud.rs` 的 SpawnType observer 中删除 `c.spawn_type = btn.0`，只保留 `cmd_buf.push(SetSpawnType{...})`。consume_commands_system 已有正确的 SetSpawnType 处理逻辑。

### D3: P2 trait 注入

render_view 中的现有系统按类型迁移：

| 当前模式 | 迁移目标 | 数量 |
|---------|---------|------|
| `NonSendMut<SimulationWorld>` 只读 | `NonSend<SimulationWorld>` + `query()` | ~15 处 |
| `NonSendMut<SimulationWorld>` + `cmd_buf.push` | `NonSend<SimulationWorld>` + `ResMut<CommandBuffer>.push()` | 9 处（注意：`submit_command()` 推入错误 buffer 已修复） |
| `NonSendMut<SimulationWorld>` 混合 | 拆分 Reader + cmd_buf.push | 0 处（P1 已消除） |

### D4: P3 CommandSource 统一

```rust
pub trait CommandSource {
    fn commands_for_tick(&mut self, tick: u32, ctx: &DriverContext) -> Vec<GameCommand>;
    fn total_ticks(&self) -> Option<u32>; // Finite = Some, Streaming = None
}
```

消除 `handle_seek` 和 `simulation_driver_system` 中对 `CommandSource::Replay` 内部字段的直接访问。在 `handle_seek` 中：`let max = source.total_ticks().unwrap_or(u32::MAX);`

## Risks / Trade-offs

- **[P2 迁移范围]** render_view 中 ~15 处系统需改为 `Res<SimulationReader>` → 批量替换，测试覆盖后再合并
- **[API 扩散]** `SimulationReader` 和 `CommandSink` 是 bevy_adapter 对外公开 API，一旦发布后修改成本高 → 保持最小接口，`query_world(|w|)` 而非逐功能方法
- **[P3 handle_seek 重构]** 当前直接访问 `source.replay.total_ticks` → 先加 `total_ticks()` 方法，再消除直接访问

## Migration Plan

```
P1 — 消除绕过 Command Pipeline 的直接修改
 ├ 改动: SpawnType observer 删除直接修改，只留命令
 ├ Exit Criteria: cargo check + test_driver_live_replay_determinism pass
 └ 进入 P2

P2 — 编译期 Guard（SimulationReader + CommandSink）
 ├ 改动: 定义 trait; render_view 迁移所有 NonSendMut 到 Res<SimulationReader>
 ├ Exit Criteria: render_view 无 NonSendMut<SimulationWorld>; cargo check; Architecture Test 全绿
 └ 进入 P3

P3 — CommandSource 统一
 ├ 改动: handle_seek 不访问内部字段; 消除 is_replay(); 用 total_ticks()
 ├ Exit Criteria: driver 测试通过; cargo check
 └ Merge

P4 — 架构测试（并行）
 ├ Architecture Tests + Determinism Tests
 └ 贯穿全程
```

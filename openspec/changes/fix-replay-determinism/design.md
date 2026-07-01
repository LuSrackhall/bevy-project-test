## Context

参考 brainstorm-spec.md 获取完整的高层设计（Goals/Non-Goals/Decisions/Risks）。本文件聚焦实作层面的技术细节。

当前 replay DESYNC 检测机制在 `bevy_adapter::driver` 中：每 20 tick 用 `golden_test::hash_world_state` 对比 hash，发现不一致时 log ERROR。该机制能告警但无法定位根因。

## Goals / Non-Goals

**Goals:**
- 实现三步定位法（精确分歧 tick → driver 集成测试 → phase 扩散追踪）
- 新增 driver 层集成测试 `test_driver_live_replay_determinism` 作为回归防护
- 根据诊断结果修复根因（分支修复策略见 brainstorm-spec.md D2）

**Non-Goals:**
见 brainstorm-spec.md Goals / Non-Goals

## Decisions

### D1: Driver 层集成测试设计 — 已执行

位置：`crates/bevy_adapter/src/driver.rs` 现有 test 模块内。

已创建三个测试：
1. `test_driver_live_replay_determinism` — Live→ReplayFile→Replay, 5000 tick (覆盖用户 DESYNC tick 4040+), AI + 多时段命令, N=5000
2. `test_replay_seek_continuation_determinism` — forward seek to midpoint + continuation
3. `test_replay_backward_seek_determinism` — backward seek (reinit) + forward replay

**诊断结论：三个测试全部通过 ✅**

这意味着：仿真层（`run_tick_default`）+ 命令注入路径（`commands_for_tick → inject_commands → run_tick_default`）+ seek 路径全部是确定性的。根因不在 simulation 层或 driver 的 tick-by-tick 处理中。

剩余疑点：
- 20+ 个 render_view 系统通过 `NonSendMut<SimulationWorld>` 读取世界（只读），但 bevy 跨帧调度顺序可能有未知影响
- `time.delta_secs() * speed` 的 accumulator 累积在不同帧率下产生不同批次大小，但总 tick 数确定
- 特定 replay 文件在 tick 4040 附近的 Entity 组合可能触发仿真边缘情况

### D2: 诊断增强

在 DESYNC 检测时增加差异化日志：记录分歧 tick 的 entity 数、总 HP、城市数差异。此改动作长期诊断，不修改 hash 检测逻辑。

### D3: Phase 扩散追踪（备用）

如果前述诊断仍无法定位，对第一个分歧 tick 在 `run_tick` 每个子系统 phase 后插临时 hash。测试通过后暂不执行。

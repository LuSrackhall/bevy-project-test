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

### D1: Driver 层集成测试设计

位置：`crates/bevy_adapter/src/driver.rs` 现有 test 模块内。

结构：
1. **录制阶段**：新建 `World` + `SimulationDriver::new_live()`，驱动 N tick（带 AI + 模拟人工命令）
2. **构建 ReplayFile**：手动构造 `ReplayFile`，记录命令和每 tick hash
3. **回放阶段**：新建 `World` + `SimulationDriver::new_replay()`，驱动相同 N tick
4. **验证**：`assert_eq!(hash_live, hash_replay)`

关键：使用 `SimulationDriver` 的 `source.commands_for_tick()` + `inject_commands()` + `simulation::run_tick_default()` 路径（而非直接调用 `run_tick_default`），与真实 driver 流程一致。

### D2: 精确分歧点定位

创建临时提交或 feature flag，将 `DESYNC_CHECK_INTERVAL` 替换为 1。
在 `ReplayFile::tick_hashes` 中每 tick 记录一个 hash。
回放时一旦首次检测到 hash 不等，立即 log 该 tick 并停止回放（避免大量重复输出）。

所有 hash 相关改动在诊断完成后回退。

### D3: Phase 扩散追踪

在 `simulation::lib.rs::run_tick` 中，在每个 phase 后加入临时的 hash_world_state 调用。
将结果通过 log 输出。

```
Phase 前 → hash0
consume_commands 后 → hash1
combat_engagement 后 → hash2
facing_turn 后 → hash3
soldier_movement 后 → hash4
...全部 phase...
AI 后 → hashN
```

对比 Live 录制和 Replay 回放的同一 tick 各 phase hash，首个差异 phase 即为根因所在系统。

为提高效率，仅对第一个分歧 tick 做 phase 追踪，非所有 tick。

### D4: 修复实施

见 brainstorm-spec.md D2。具体 code change 取决于诊断结果。

### D5: 录制重构（如诊断需要）

`ReplayRecorder::record_tick` 移除 `if !commands.is_empty()` 过滤：
```rust
pub fn record_tick(&mut self, tick: u32, commands: &[GameCommand]) {
    if self.is_recording {
        self.command_log.push((tick, commands.to_vec()));
    }
}
```
同时 `ReplayFile::record_tick` 保持 `!commands.is_empty()` 过滤（文件格式优化）。
回放时从 `command_log`（全量 tick）构建 `ReplayFile::commands_per_tick`（仅含非空命令）。

## Risks / Trade-offs

- **[诊断串行] Driver 集成测试通过后才能确定是 bevy 层问题** → 如果测试失败则直接定位仿真层，加速诊断
- **[hash 碰撞] hash_world_state 使用 FNV-1a 64bit，碰撞概率极低** → 可接受
- **[H1: 非确定性 RNG 消耗]** AI 决策可能在某处因 Entity 顺序变化多调用一次 `rng.next_u64()`，导致后续所有 RNG 调用产生不同输出 → 修复方案：在 AI 层前置校正或按 UnitId 排序 Entity 遍历

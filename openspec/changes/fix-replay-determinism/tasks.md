## 1. 录制路径重构 — 无条件录制

- [x] 1.1 `ReplayRecorder::record_tick` 移除 `!commands.is_empty()` 过滤，改为记录所有 tick
- [x] 1.2 `ReplayFile::record_tick` 保持非空过滤（文件格式优化不变）
- [x] 1.3 验证：空命令 tick 被记录为空 Vec，回放时 `commands_for_tick` 返回空 Vec（代码片审确认：`command_log` 记录空 Vec，`finish`→`ReplayFile::record_tick` 过滤非空，`commands_for_tick` 对未记录 tick 返回 `&[]`）

## 2. Driver 层集成测试

- [x] 2.1 在 `bevy_adapter/src/driver.rs` tests 模块中新建 `test_driver_live_replay_determinism`
- [x] 2.2 测试覆盖 5000 tick（AI + 多时段模拟人工命令），覆盖至用户 DESYNC 的 tick 4040 之后
- [x] 2.3 验证：测试 **通过** → 仿真层+命令注入路径确定性的
- [x] 新增 `test_replay_seek_continuation_determinism` — forward seek 确定性的 ✅
- [x] 新增 `test_replay_backward_seek_determinism` — backward seek + reinit 确定性的 ✅

## 3. 架构修复：双 seek 处理冲突

- [ ] 3.1 诊断确认：`replay_seek_system`(render_view) 和 `simulation_driver_system::handle_seek`(bevy_adapter) 各自独立处理 `async_seek`，无明确调度顺序，可能同时处理 seek 导致 World 被处理两次
- [ ] 3.2 修复：移除 `driver.rs::handle_seek` 中的 seek 处理逻辑，入口统一到 `replay_seek_system`（保持 `simulation_driver_system` 中 `async_seek` 的 early-return guard）
- [ ] 3.3 验证：`replay_seek_system` 和 `simulation_driver_system` 不再冲突处理 seek

## 4. Phase 扩散追踪（仅在第一个分歧 tick 执行）

- [ ] 4.1 在 `simulation::lib.rs::run_tick` 中每个子系统 phase 后插入临时 hash_world_state
- [ ] 4.2 对比 Live 录制和 Replay 回放在同一 tick 各 phase hash，定位首个差异 phase

## 5. 根因修复（根据诊断结果）

- [ ] 5.1 如果根因是 HashMap 迭代非确定：替换 `combat/mod.rs` 和 `soldier/mod.rs` 中影响状态的 HashMap 为 BTreeMap
- [ ] 5.2 如果根因是命令注入时序差异：重构 inject_commands → take_for_tick 路径
- [ ] 5.3 如果根因是 AI RNG 消耗分歧：在 AI 层按 UnitId 排序 Entity 遍历，确保 RNG 消耗量一致
- [ ] 5.4 如果根因是 World::query 迭代顺序变化：所有影响仿真状态的系统遍历改为按 UnitId 排序
- [ ] 5.5 如果根因在 bevy 帧时序层：在 simulation_driver_system 和 handle_seek 中增加 tick 边界校验

## 6. 验证与回归

- [ ] 6.1 回退所有诊断改动（hash 频率还原为 20 tick，移除临时 phase hash）
- [ ] 6.2 `cargo test --package simulation` 全量通过
- [ ] 6.3 `cargo test --package bevy_adapter` 全量通过（含 2.1 新增的 driver 集成测试）
- [ ] 6.4 `cargo build --release` 无错误
- [ ] 6.5 手动验证：录制一场含操作的对局，回放至结束，确认零 DESYNC

---

## Post-Implementation Workflow

<!-- DO NOT MODIFY THIS SECTION — it defines the required workflow after all tasks are complete -->

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

## 1. 录制路径重构 — 无条件录制

- [x] 1.1 `ReplayRecorder::record_tick` 移除 `!commands.is_empty()` 过滤，改为记录所有 tick
- [x] 1.2 `ReplayFile::record_tick` 保持非空过滤（文件格式优化不变）
- [x] 1.3 验证：空命令 tick 被记录为空 Vec，回放时 `commands_for_tick` 返回空 Vec

## 2. Driver 层集成测试

- [x] 2.1 `test_driver_live_replay_determinism` — 15000 tick, seed 42, Small
- [x] 2.2 `test_driver_live_replay_determinism_medium` — 10000 tick, seed 99, Medium
- [x] 2.3 `test_driver_live_replay_determinism_seed_77` — 15000 tick, seed 77, Small
- [x] 2.4 `test_replay_seek_continuation_determinism` — forward seek + continuation
- [x] 2.5 `test_replay_backward_seek_determinism` — backward seek + reinit

## 3. 根因修复：SpawnType 录制遗漏（真实 DEYNC 根因）

- [x] 3.1 诊断确认：spawn type observer 直接写 `c.spawn_type = btn.0` 但不推命令，所有播放文件不记录此修改。Replay 时城市产出默认兵种 → 战斗结果不同 → hash 分歧
- [x] 3.2 修复：observer 推 `GameCommand{ SetSpawnType }` 到 `cmd_buf`，同时保持直接修改（即时反馈）
- [x] 3.3 验证：observer 同时执行直接修改 + 命令推入，新 replay 文件录制 SetSpawnType

## 4. 根因修复：Replay 越界 ghost ticks

- [x] 4.1 诊断确认：`total_ticks` 处只有注释无操作，回放越过录制终点继续模拟
- [x] 4.2 修复：`driver.scheduler.is_paused = true` 在 `current_tick >= total_ticks` 时触发
- [x] 4.3 `handle_seek` 中 cap seek target 为 `replay.total_ticks`

## 5. 构建与验证

- [x] 5.1 `cargo check --package render_view` 通过
- [x] 5.2 `cargo check --package bevy_adapter` 通过
- [x] 5.3 `cargo test --package bevy_adapter -- test_driver` 3 test passed
- [x] 5.4 响应式确认：用户确认 DESYNC 问题已解决

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

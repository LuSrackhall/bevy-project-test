## 1. P1 — 消除绕过 Command Pipeline 的直接修改

- [x] 1.1 SpawnType observer：删除 `c.spawn_type = btn.0` 直接修改，只保留 `cmd_buf.push(SetSpawnType)`
- [x] 1.2 全局审查 render_view 中所有 `NonSendMut<SimulationWorld>` 取值点，确认无遗漏的绕过路径（P1 范围验证通过：仅 1 处 SpawnType 直接修改，已消除）
- [x] 1.3 `cargo check --package render_view` 通过
- [x] 1.4 `cargo test -p bevy_adapter -- test_driver` 全量通过（确定性测试回归）

## 2. P2 — 编译期 Guard：SimulationReader + CommandSink

- [x] 2.1 在 `bevy_adapter` 中定义 `SimulationReader` trait（`query_world(|&World|)`）
- [x] 2.2 在 `bevy_adapter` 中定义 `CommandSink` trait（`submit_command(GameCommand)`）
- [x] 2.3 `SimulationWorld` 内部改为 `UnsafeCell` + `world_ref()` / `world_mut()` / `query()` API，render_view 通过公共 API 访问
- [x] 2.4 通过 `SimulationWorld` 的公共方法暴露 `SimulationReader::query_world()` / `CommandSink::submit_command()`
- [x] 2.5 render_view 中所有只读系统（update_top_bar、selection 系统、camera、debug_shape、unit_info_bar 等 ~15 处）改为 `NonSend<SimulationWorld>` + `sim.query()` / `sim.world_ref()`
- [x] 2.6 render_view 中所有命令下发系统（observer 回调 3 处）改为通过 `CommandSink::submit_command()`
- [x] 2.7 render_view 不再有 `NonSendMut<SimulationWorld>` 导入（所有只读系统使用 `NonSend`，命令下发使用 `submit_command()`）
- [x] 2.8 `cargo check --package render_view` 通过
- [x] 2.9 `cargo test -p bevy_adapter -- test_driver` 全量通过

## 3. P3 — CommandSource 统一

- [x] 3.1 `CommandSource` trait 新增 `total_ticks() → Option<u32>`，保留 `is_live()`（用于录制逻辑）
- [x] 3.2 `handle_seek` 改用 `source.total_ticks()` 替代直接 match `CommandSource::Replay`
- [x] 3.3 `simulation_driver_system` 结束处的 total_ticks 检查改用 `source.total_ticks().unwrap()`
- [x] 3.4 `cargo check --package bevy_adapter` 通过
- [x] 3.5 `cargo test -p bevy_adapter -- test_driver` 全量通过

## 4. P4 — 架构测试 + 文档

- [x] 4.1 新增架构测试：render_view crate 不得使用 `world_mut()` 或直接导入 `use simulation::World`
- [x] 4.2 验证宪法 v1.1 条款与实现一致（§1.2.7 / §2.5.4 / §2.5.5）
- [x] 4.3 `cargo test --package simulation` 全量通过
- [x] 4.4 `cargo test --package bevy_adapter` 全量通过（含确定性测试 + 架构测试）
- [x] 4.5 `cargo build --release` 无错误

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

## 1. 创建 relay_core 模块

- [x] 1.1 新建 `crates/bevy_adapter/src/relay_core.rs`，定义 `RelayConfig`、`RelayCtx`、`run_relay()`、`handle_client()`、`relay_write()`
- [x] 1.2 从 `crates/relay/src/lib.rs` 迁移 handle() 逻辑到 `handle_client()`（~150 行），删除死匹配臂和 `next_player_id` 字段
- [x] 1.3 注册模块到 `crates/bevy_adapter/src/lib.rs`（`mod relay_core`）
- [x] 1.4 编译验证（`cargo build -p bevy_adapter`）

## 2. 适配 ThreadRelayRuntime

- [x] 2.1 修改 `run_local_relay()`，在 TCP bind + port_tx.send + UDP beacon 后调用 `relay_core::run_relay(listener, config, stop)` 替代空转循环
- [x] 2.2 编译验证（`cargo build -p bevy_adapter`）

## 3. 适配 relay crate

- [x] 3.1 修改 `relay::start_relay()` 为薄包装：bind listener → 构造 RelayConfig → 调用 `relay_core::run_relay()`
- [x] 3.2 删除 relay crate 中已迁移到 relay_core 的代码（handle、RelayCtx、relay_write）
- [x] 3.3 编译验证（`cargo build -p relay`）

## 4. 验证

- [ ] 4.1 现有测试通过（`cargo test -p bevy_adapter --lib`）
- [ ] 4.2 relay 集成测试通过（`cargo test -p relay`）

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

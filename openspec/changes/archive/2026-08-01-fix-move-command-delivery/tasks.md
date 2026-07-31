## 1. 实现修复

- [x] 1.1 `SimulationDriver::command_delay()` 方法
- [x] 1.2 `command_issue_system` 用 command_delay 替代硬编码 +1
- [x] 1.3 `seek_stance_shortcut_system` 用 command_delay
- [x] 1.4 `network_flush_system` 窗口发送
- [x] 1.5 输入系统 `.before(network_flush_system)`

## 2. 测试

- [x] 2.1 新增 `network_move_e2e.rs` 可靠性测试
- [x] 2.2 `cargo test` 全部通过
- [x] 2.3 `cargo check` 编译通过

---

## Post-Implementation Workflow

<!-- DO NOT MODIFY THIS SECTION -->

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

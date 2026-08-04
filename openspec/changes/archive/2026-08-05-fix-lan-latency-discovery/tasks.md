## 1. TCP_NODELAY

- [x] 1.1 `transport.rs` 客户端 connect 两处加 set_nodelay(true)
- [x] 1.2 `relay_core.rs` accept 后加 set_nodelay(true)

## 2. 子网广播

- [x] 2.1 `session_host/thread.rs` beacon 发送到 /24 子网广播地址

## 3. 验证

- [x] 3.1 `cargo check` 编译通过
- [x] 3.2 全量测试通过

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

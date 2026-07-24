## 1. 删除冗余 beacon

- [x] 1.1 删除 `crates/relay/src/lib.rs` 中 30-59 行 beacon 代码及不再使用的 imports
- [x] 1.2 编译验证（`cargo build -p relay`）
- [x] 1.3 测试验证（`cargo test -p relay` — 3 个预存失败不受影响；`cargo test -p bevy_adapter --lib` — 30/30 通过）

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

## 1. 客户端 drain

- [x] 1.1 `network_flush_system` 中 `Res<CommandBuffer>` → `ResMut<CommandBuffer>`
- [x] 1.2 替换 `iter().filter().cloned().collect()` 为 `take_for_tick(cmd_tick)`

## 2. Relay 覆盖

- [x] 2.1 `buffer[tick][player_id].extend(...)` → 覆盖赋值

## 3. 验证

- [x] 3.1 `cargo check` 编译通过
- [x] 3.2 `cargo test` 仿真测试通过

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

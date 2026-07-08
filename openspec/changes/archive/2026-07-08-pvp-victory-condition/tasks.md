## 1. 重写 check_victory_system

- [x] 1.1 添加 `use simulation::types::PlayerSlots;` 导入（如需要）
- [x] 1.2 重写 `check_victory_system`：`LocalPlayerId` + `PlayerSlots` 过滤

## 2. 编译验证

- [x] 2.1 运行 `cargo check -p render_view` 确认编译通过
- [x] 2.2 运行 `cargo test -p simulation` 确认全部通过

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run myspec-verify to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run openspec archive on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

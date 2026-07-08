## 1. 修复 debug_shape 颜色映射

- [ ] 1.1 添加 `let lid = crate::local_player_id(&*sim_world);`
- [ ] 1.2 添加 `faction_is_player_or_enemy` 辅助函数
- [ ] 1.3 替换 4 处 match → if-else

## 2. 编译验证

- [ ] 2.1 运行 `cargo check -p render_view` 确认编译通过
- [ ] 2.2 运行 `cargo test -p simulation` 确认全部通过

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run myspec-verify
2. **User Acceptance**: Present change summary
3. **Merge**: After user accepts, merge to main
4. **Archive**: openspec archive on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

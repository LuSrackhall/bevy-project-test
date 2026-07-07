## 1. 修复 HUD 显示层 FactionId 硬编码

- [ ] 1.1 在 `update_top_bar` 中添加 `let lid = crate::local_player_id(&*sim_world);`
- [ ] 1.2 修复 line 625：`FactionId(0)` → `FactionId(lid)`
- [ ] 1.3 修复 match arm（line 656）：改为 guard 匹配，简化标签
- [ ] 1.4 在 `seek_panel_count_system` 中添加 `let lid = crate::local_player_id(&*sim_world);`
- [ ] 1.5 修复 line 1181 和 1208：`FactionId(0)` → `FactionId(lid)`

## 2. 编译验证

- [ ] 2.1 运行 `cargo check -p render_view` 确认编译通过
- [ ] 2.2 运行 `cargo test -p simulation` 确认全部通过

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run myspec-verify to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run openspec archive on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

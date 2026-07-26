## 1. 实现房间名输入

- [x] 1.1 `open_create_room_modal` 中 `ModalRoomName` 按钮 → `EditableText` + `AutoFocus` + `ModalRoomName` 标记
- [x] 1.2 创建按钮 observer 改为 `Query<&EditableText, With<ModalRoomName>>` 读取输入值
- [x] 1.3 编译验证

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

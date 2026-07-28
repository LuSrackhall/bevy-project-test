## 1. 新增标记组件

- [x] 1.1 定义 `LanLobbyRowData(RelayId)` 组件
- [x] 1.2 定义 `RoomNameLabel`, `MapLabel`, `PlayersLabel`, `StateLabel` Text 标记组件

## 2. 增量更新逻辑

- [x] 2.1 重写 `update_room_list`：处理 `existing_rows` 匹配逻辑
- [x] 2.2 实现移除消失行：对比 `servers` 后 despawn 不存在的行
- [x] 2.3 实现添加新行：spawn 新行（含 `WidgetButton` + `On<Activate>` observer）
- [x] 2.4 实现存量行文本更新：通过标记组件修改 Text 内容
- [x] 2.5 对 `servers.servers` 按 `relay_id` 排序

## 3. 清理与验证

- [x] 3.1 移除 rc9 的 `On<Pointer<Click>>` Text observer 代码
- [x] 3.2 验证编译：`cargo check -p render_view`
- [x] 3.3 验证全项目编译：`cargo check`

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

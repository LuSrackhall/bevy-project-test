## 1. 移除 Popover，改为手动定位

- [x] 1.1 重写 observer：通过 `ChildOf` 从触发按钮找到 anchor，查询 `UiGlobalTransform` 和 `ComputedNode` 获取位置信息
- [x] 1.2 手动计算 popup 位置：根据 trigger 位置和窗口高度决定弹出方向（上方优先，下方备选）
- [x] 1.3 移除 `Popover` 组件和 `PopoverPlacement` 相关导入
- [x] 1.4 恢复 popup 的 `BackgroundColor`（正常不透明颜色）

## 2. 修复 popup 父节点关系

- [x] 2.1 将 popup 添加到 `ev.source`（触发按钮）而非 anchor，确保关闭逻辑正确
- [x] 2.2 更新 observer 查询以通过 `q_trigger` 获取触发按钮的子节点来查找已有 popup

## 3. 清理

- [x] 3.1 移除 `PopoverReady` 和 `MenuTextMarker` 标记组件
- [x] 3.2 移除 `reveal_popover` 系统及其注册
- [x] 3.3 移除未使用的 Popover 相关导入

## 4. 验证

- [x] 4.1 运行 `cargo build` 确保编译通过
- [x] 4.2 运行游戏，确认：展开无闪烁、可正常收起、宽度正确

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/fix-popover-flicker`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

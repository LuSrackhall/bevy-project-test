## 1. 修复开始联机按钮 Query

- [x] 1.1 将 `Query<(&NetworkRelayAddrInput, &NetworkPlayerCount, &NetworkPlayerId)>` 拆为三个独立 Query，每项值独立获取并回退默认值
- [x] 1.2 验证：开始联机按钮观察者正确读取到各兄弟实体的配置组件值

## 2. 实现玩家数量（Count）按钮交互

- [x] 2.1 为 `NetworkPlayerCount` 按钮添加 `.observe(|_ev: On<Activate>, ...)`，点击循环 `2 → 3 → 4 → 2`
- [x] 2.2 同步更新按钮子 Text 显示（`children.iter().next()` + if-let 查找）
- [x] 2.3 Count 减小时自动 clamp ID：通过 `Query<(&mut NetworkPlayerId, &Children)>` 写入 ID 组件和对应 Text

## 3. 实现玩家 ID 按钮交互

- [x] 3.1 为 `NetworkPlayerId` 按钮添加 `.observe(|_ev: On<Activate>, ...)`，点击循环 `0 → 1 → … → (count-1) → 0`
- [x] 3.2 同步更新按钮子 Text 显示

## 4. 提交和验证

- [ ] 4.1 `cargo build --workspace` 编译通过
- [ ] 4.2 `cargo test --workspace` 所有测试通过

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

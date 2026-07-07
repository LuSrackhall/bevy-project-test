## 1. 提取公共辅助函数

- [x] 1.1 在 `render_view/src/lib.rs` 中添加 `pub(crate) fn local_player_id()`
- [x] 1.2 将 `selection.rs` 的私有 `local_player_id()` 改为调用公共函数
- [x] 1.3 将 `camera.rs` 的内联读取改为调用公共函数

## 2. 修复 HUD 按钮 player_id 硬编码

- [x] 2.1 修复 SpawnTypeBtn 观察者（line 287）：添加 `NonSend<SimulationWorld>` 参数，使用 `local_player_id()`
- [x] 2.2 修复 ShieldButton 观察者（line 367）：使用 `local_player_id()`（已有 `NonSendMut`）
- [x] 2.3 修复 SeekIssueBtn 观察者（line 505/510）：使用 `local_player_id()`（已有 `NonSendMut`）

## 3. 删除死代码

- [x] 3.1 删除 `render_view/src/session.rs`
- [x] 3.2 从 `render_view/src/lib.rs` 移除 `pub mod session`

## 4. 回归测试

- [x] 4.1 在 `simulation/src/command.rs` 添加 `test_local_player_id_fallback` 测试

## 5. 编译验证

- [x] 5.1 运行 `cargo test -p simulation` 确认全部通过
- [x] 5.2 运行 `cargo test` 确认全量 137 测试通过
- [x] 5.3 运行 `cargo clippy` 确认无新增警告

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

## 1. Fix A: GameJoined → NetworkCommandSource

- [x] 1.1 在 `lobby_update_system` 中处理 `NetworkEvent::GameJoined`，更新 `SimulationDriver.source.player_id`
- [x] 1.2 更新 `LocalPlayerIdentity` Resource
- [x] 1.3 编译验证 + 确认 CLI 路径不受影响

## 2. Fix B: validate_commands

- [x] 2.1 在 `simulation/src/lib.rs` 中实现 `validate_commands()` 函数
- [x] 2.2 在 `run_tick()` 的命令排序后、`consume_commands_system` 前调用
- [x] 2.3 验证单人模式下 `PlayerSlots` 不存在时的兼容性
- [x] 2.4 编译验证（121 个仿真测试全部通过）

## 3. ADR

- [x] 3.1 创建 `docs/adr/0007-command-envelope.md`，记录 Command Envelope 的架构说明

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

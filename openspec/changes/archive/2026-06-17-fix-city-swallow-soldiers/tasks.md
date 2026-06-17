## 1. 修复城池吞兵 bug

- [x] 1.1 在 `simulation/src/soldier/mod.rs` 的 `city_interaction_system` 中，将 554-555 行的 despawn + origin_decrement 逻辑移入治愈/升级条件分支内，引入 `consumed` 标志守卫
- [x] 1.2 验证满血满级城池不再吞兵：士兵到达后停在原位，不被 despawn，不扣减出生城人口

## 2. 修复攻击目标消失后移动意图丢失 bug

- [x] 2.1 在 `simulation/src/combat/mod.rs` 第 131 行，将 `command_target: None` 改为 `command_target: sd.cmd_target`
- [x] 2.2 验证士兵在攻击目标消失后恢复向原 `command_target` 移动

## 3. 测试验证

- [x] 3.1 运行 `cargo test -p simulation` 确认无回归
- [x] 3.2 运行 `cargo build` 确认编译通过

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/fix-city-swallow-soldiers`

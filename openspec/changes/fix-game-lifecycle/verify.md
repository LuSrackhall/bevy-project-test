## Verification Report: fix-game-lifecycle

### Completeness

| 检查项 | 状态 |
|--------|------|
| tasks.md 完成率 | 26/26 (100%) |
| game-lifecycle spec 需求覆盖 | 6/6 |
| pause-system spec 需求覆盖 | 6/6 |
| bevy-adapter-crate spec 需求覆盖 | 3/3 |
| render-view-crate spec 需求覆盖 | 3/3 |
| game-over-panel spec 需求覆盖 | 1/1 |
| ui-system-fixes spec 需求覆盖 | 2/2 |

### Correctness

| 需求 | 实现证据 | 状态 |
|------|----------|------|
| GameState 3 变体 | `render_view/src/lib.rs:16` — MainMenu/Playing/GameOver | PASS |
| Paused 资源 | `bevy_adapter/src/lib.rs:19` — Paused(bool) 在 bevy_adapter 中 | PASS |
| NeedsGameReset 标志 | `render_view/src/lib.rs:25` | PASS |
| tick/sync 守卫 | `bevy_adapter/src/lib.rs:36` — GameActive AND NOT Paused | PASS |
| reset_game_system | `render_view/src/lib.rs:92` — OnEnter(Playing) 完整重置 | PASS |
| cleanup_playing | `render_view/src/lib.rs:160` — OnExit(Playing) 销毁 HUD | PASS |
| 暂停 UI 可见性驱动 | `render_view/src/ui/mod.rs:44` — update_pause_visibility | PASS |
| 暂停按钮行为 | `pause.rs:22` — Resume/Restart/Menu 正确操作资源 | PASS |
| GameOver "再来一局" | `gameover.rs:21` — NeedsGameReset(true) + MainMenu | PASS |
| "单人模式"按钮 | `menu.rs:22` — NeedsGameReset(true) + Playing | PASS |
| SimulationWorld 延迟初始化 | `main.rs:23` — 空 SimulationWorld(seed=0) | PASS |
| UnitIdMapper.clear() | `bevy_adapter/src/mapper.rs:34` | PASS |
| SelectionState.clear() | `render_view/src/selection.rs:50` | PASS |

### Coherence

| 设计决策 | 实现一致性 | 备注 |
|----------|-----------|------|
| 单状态枚举 + Paused 资源 | PASS | Paused 从 render_view 移至 bevy_adapter（必要修正） |
| 暂停时仿真停止 tick | PASS | 守卫 `GameActive AND NOT Paused` |
| 重开经 MainMenu 中转 | PASS | 所有重开按钮设 NeedsGameReset + MainMenu |
| Esc 立即暂停 | PASS | 简化为无条件 paused.0 = true |
| OnEnter/OnExit 生命周期 | PASS | reset_game_system.before(setup_hud) |

### Issues

- **SUGGESTION**: Paused 资源位置从设计的 render_view 移至 bevy_adapter。这是正确的架构决策（bevy_adapter 需要直接访问 Paused 来守卫 tick），但设计文档应更新。
- **SUGGESTION**: handle_pause_input 行为从设计的"先清选区再暂停"简化为"无条件暂停"。UX 改进，设计文档应更新。
- **WARNING**: 编译有 22 个 warnings（未使用变量/导入），均为已有代码，非本次变更引入。

### 结论

所有需求已正确实现，实现与设计基本一致（两处合理偏离已记录）。编译通过，83 个测试全部通过，4 项手动验证全部通过。

## 1. 状态枚举与资源定义

- [x] 1.1 修改 `render_view/src/lib.rs`：`GameState` 枚举从 4 变体改为 3 变体（移除 `Paused`），添加 `Paused(bool)` 和 `NeedsGameReset(bool)` 资源
- [x] 1.2 修改 `bevy_adapter/src/mapper.rs`：`UnitIdMapper` 添加 `clear()` 方法

## 2. 系统守卫

- [x] 2.1 修改 `bevy_adapter/src/lib.rs`：`tick_driver_system` 和 `sync_entities_system` 加 `run_if(in_state(GameState::Playing).and(not_paused))`
- [x] 2.2 修改 `bevy_adapter/src/lib.rs`：移除 `Startup` 阶段的 `backfill_entities_system` 注册
- [x] 2.3 修改 `render_view/src/lib.rs`：`check_victory_system` 和所有 gameplay 系统加 `not_paused` 守卫
- [x] 2.4 修改 `render_view/src/ui/mod.rs`：HUD 更新系统加 `not_paused` 守卫
- [x] 2.5 修改 `render_view/src/selection.rs`：选择和指令系统加 `not_paused` 守卫

## 3. 生命周期系统

- [x] 3.1 在 `render_view/src/lib.rs` 中实现 `reset_game_system`：销毁旧实体、清空状态、用随机种子重建 SimulationWorld
- [x] 3.2 在 `render_view/src/lib.rs` 中实现 `cleanup_playing` 系统（`OnExit(Playing)` 销毁 HUD + 暂停 UI）
- [x] 3.3 注册 `OnEnter(Playing)` → `reset_game_system`，`.before(setup_hud)`
- [x] 3.4 注册 `OnExit(Playing)` → `cleanup_playing`
- [x] 3.5 修改 `setup_hud`：添加 `.after(reset_game_system)` 保证执行顺序

## 4. 暂停系统

- [x] 4.1 修改 `render_view/src/ui/pause.rs`：暂停菜单按钮改为操作 `Paused` 资源和 `NeedsGameReset` 标志
- [x] 4.2 修改 `render_view/src/ui/mod.rs`：`handle_pause_input` 改为操作 `Paused` 资源而非 `NextState`
- [x] 4.3 在 `setup_hud` 中创建暂停菜单 UI（初始 `Visibility::Hidden`）
- [x] 4.4 实现 `update_pause_visibility` 系统，根据 `Paused` 资源切换暂停菜单可见性

## 5. 按钮行为更新

- [x] 5.1 修改 `render_view/src/ui/menu.rs`："单人模式"按钮设 `NeedsGameReset(true)` + `NextState(Playing)`
- [x] 5.2 修改 `render_view/src/ui/gameover.rs`："再来一局"按钮设 `NeedsGameReset(true)` + `NextState(MainMenu)`
- [x] 5.3 修改 `render_view/src/ui/pause.rs`："重新开始"按钮设 `Paused(false)` + `NeedsGameReset(true)` + `NextState(MainMenu)`

## 6. main.rs 修改

- [x] 6.1 修改 `src/main.rs`：移除 `init_sim_world()` 函数和 `insert_non_send_resource(init_sim_world())`，改为插入空 SimulationWorld

## 7. 编译与验证

- [x] 7.1 编译通过（`cargo build`）
- [x] 7.2 所有现有测试通过（`cargo test`）
- [ ] 7.3 手动验证：主菜单不启动仿真，点击"单人模式"后游戏正常开始
- [ ] 7.4 手动验证：游戏结束后"再来一局"能正确重开新游戏
- [ ] 7.5 手动验证：暂停/继续功能正常（Esc 暂停，继续按钮恢复）
- [ ] 7.6 手动验证：暂停中"重新开始"能正确重开新游戏

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/fix-game-lifecycle`

## Why

游戏存在两个阻断性 bug：(1) 仿真从 app 启动就开始运行，主菜单是假的——玩家在菜单停留多久，游戏就跑了多久；(2) 游戏结束后无法重开——"再来一局"和"重新开始"只切换 UI 状态，不重置仿真世界，导致 `check_victory_system` 立即再次触发 GameOver 死循环。根因是仿真生命周期与 UI 状态混在同一个 `GameState` 枚举中，且缺乏生命周期钩子管理仿真世界的初始化和销毁。

## What Changes

- **BREAKING**: `GameState` 枚举从 4 变体（`MainMenu`/`Playing`/`Paused`/`GameOver`）改为 3 变体（`MainMenu`/`Playing`/`GameOver`），`Paused` 移出状态机
- 新增 `Paused(bool)` 资源替代 `GameState::Paused` 状态变体，暂停/继续不再触发 `OnEnter`/`OnExit`
- 新增 `NeedsGameReset(bool)` 资源，区分首次开始游戏和从暂停恢复
- `tick_driver_system` 和 `sync_entities_system` 加 `run_if` 守卫，仅在 `Playing` 且未暂停时运行
- 新增 `reset_game_system`（`OnEnter(Playing)`）：销毁旧实体、清空状态、用随机种子重建仿真世界
- 新增 `cleanup_playing`（`OnExit(Playing)`）：销毁 HUD 和暂停菜单
- GameOver "再来一局"和暂停"重新开始"改为先回 MainMenu，用户再点"单人模式"开始新游戏
- 暂停 UI 改为基于 `Paused` 资源的可见性切换，不再依赖状态转换

## Capabilities

### New Capabilities
- `game-lifecycle`: 游戏状态机管理——GameState 定义、OnEnter/OnExit 生命周期钩子、reset_game_system、NeedsGameReset 标志、系统守卫条件
- `pause-system`: 暂停系统——Paused 布尔资源、暂停/继续逻辑、暂停 UI 可见性、Escape 键处理

### Modified Capabilities
- `game-over-panel`: GameOver 按钮行为变更——"再来一局"改为回 MainMenu 而非直接进 Playing
- `bevy-adapter-crate`: tick_driver_system 和 sync_entities_system 加 run_if 守卫；移除 Startup 阶段的 backfill_entities_system
- `render-view-crate`: GameState 枚举变更、RenderViewPlugin 系统注册变更
- `ui-system-fixes`: HUD 更新系统加 not_paused 守卫；setup_hud 中创建暂停菜单 UI

## Impact

- **bevy_adapter**: tick.rs（守卫）、lifecycle.rs（backfill 移到 OnEnter）、mapper.rs（新增 clear 方法）、lib.rs（系统注册）
- **render_view**: lib.rs（状态枚举、reset/cleanup 系统）、ui/mod.rs（所有 run_if 和状态引用）、ui/pause.rs（按钮逻辑重写）、ui/gameover.rs（按钮逻辑变更）、ui/menu.rs（按钮设 NeedsGameReset）、ui/hud.rs（run_if 更新）、selection.rs（run_if 更新）
- **main.rs**: 移除 init_sim_world()，改为插入空 SimulationWorld
- 所有依赖 `GameState::Paused` 的代码都需要迁移

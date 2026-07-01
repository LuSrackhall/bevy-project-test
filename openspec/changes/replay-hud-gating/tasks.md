## 1. world_stats 模块

- [ ] 1.1 新建 `simulation/src/world_stats.rs`：`FactionCounts` struct + `count_factions` 函数，`BTreeMap<Faction, (u32,u32)>`
- [ ] 1.2 `FactionCounts` 提供 `soldiers(faction)`、`cities(faction)`、`total_soldiers()`、`total_cities()` 方法
- [ ] 1.3 在 `simulation/src/lib.rs` 注册 `pub mod world_stats;`
- [ ] 1.4 编写 world_stats 单元测试（确定性 + 地图生成后计数）

## 2. HUD Observer GameMode 检查

- [ ] 2.1 `hud.rs` spawn type 按钮 observer：加 `game_mode: Res<GameMode>` 参数 + 首行 `if *game_mode == GameMode::Replay { return; }`
- [ ] 2.2 `hud.rs` toolbar 按钮 observer（盾牌/框选/优先移动）：加 GameMode 检查
- [ ] 2.3 `hud.rs` 索敌下发按钮 observer：加 GameMode 检查

## 3. HudInteractive 标记 + 隐藏系统

- [ ] 3.1 定义 `HudInteractive` marker component（`#[derive(Component)]`）
- [ ] 3.2 在 toolbar 容器 Node spawn 处加 `HudInteractive`
- [ ] 3.3 在索敌面板根节点 `SeekPanelRoot` spawn 处加 `HudInteractive`
- [ ] 3.4 实现 `hide_interactive_in_replay` Update 系统
- [ ] 3.5 在 `ui/mod.rs` 注册该系统，Playing state + 无 Paused gate

## 4. HUD Update 系统闸门

- [ ] 4.1 `update_top_bar` 移出原有 `.add_systems()` 块，单独注册不带 `not(GameMode::Replay)` gate
- [ ] 4.2 其余 9 个 HUD Update 系统加 `not(GameMode::Replay)` gate

## 5. update_top_bar 集成 count_factions

- [ ] 5.1 `update_top_bar` 中手动 faction 遍历替换为 `simulation::world_stats::count_factions`
- [ ] 5.2 显示文本改为动态迭代 `counts.factions.iter()`，用 `{:?}` 临时显示 Faction 名

## 6. 构建与验证

- [ ] 6.1 `cargo test --package simulation` 全量通过
- [ ] 6.2 `cargo build --release` 无错误
- [ ] 6.3 手动验证：启动回放，确认 toolbar/索敌不显示，顶部栏实时更新阵营数，底部播放器正常工作

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

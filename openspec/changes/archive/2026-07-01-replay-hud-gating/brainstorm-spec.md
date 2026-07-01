## Context

回放模式（Replay）旨在让用户观看历史对局。当前有两类问题：

1. **安全**：HUD 命令下发按钮（toolbar 的盾牌/优先移动、索敌面板的下发按钮、spawn type 切换）在回放中仍有 observer 响应，可以注入命令到 `CommandBuffer`。
2. **信息**：回放中缺少阵营兵力统计，且 HUD 顶部状态栏在回放中显示冻结的初始值。

分支代码已回退到 `73f35f3`，以下 commit 的内容需重新实施：
- Observer GameMode 检查（hud.rs 3 处）
- world_stats 模块
- HUD Update gate + 动态阵营显示

## Goals / Non-Goals

**Goals:**
- 回放中 HUD 命令下发按钮不可交互（observer GameMode 检查 + toolbar/索敌隐藏）
- 回放中 HUD 顶部状态栏实时显示阵营兵力（`count_factions`，动态 BTreeMap）
- HUD Update 系统（除 `update_top_bar` 外）回放时不执行（性能）
- Replay player 底部控制栏完整可用

**Non-Goals:**
- 不新增顶部 ReplayInfoBar（复用 HUD 顶部栏）
- 不改 `setup_hud` 签名
- 不改底部 replay player 控制栏
- 不引入 O(N) 扫描到 live 模式热路径

## Decisions

**1. world_stats.rs** — 放在 `simulation` 层。纯只读查询 `count_factions(&mut World) -> FactionCounts { factions: BTreeMap<Faction, (u32, u32)> }`。BTreeMap 保证确定性，支持未来任意多方阵营。对标 `golden_test::hash_world_state`。

**2. Observer GameMode 检查** — 在 3 个 observer 回调开头加 `if *game_mode == GameMode::Replay { return; }`：spawn type 切换、toolbar 按钮（盾牌/优先移动）、索敌下发按钮。

**3. Toolbar/索敌隐藏** — 给 toolbar 容器和索敌面板根节点加 `HudInteractive` marker。新增 `hide_interactive_in_replay` Update 系统：查 `GameMode`，回放时设 `Visibility::Hidden`。不改 `setup_hud` 签名（避免 OnEnter 中 commands 刷新时序不确定性）。

**4. HUD Update 闸门** — `ui/mod.rs` 中 HUD Update 系统加 `not(GameMode::Replay)` gate。**例外**：`update_top_bar` 不加 gate，回放中也运行，实时显示阵营统计。

**5. update_top_bar 改用 count_factions** — 将已有的手动 faction 遍历替换为 `simulation::world_stats::count_factions`，支持动态阵营显示。阵营名称使用中文（玩家/敌人/中立），不引入 i18n。

**6. 不新增顶部 InfoBar** — HUD 顶部栏已覆盖时间+城/兵/敌统计。回放中 `update_top_bar` 继续运行即可实时更新。

## Risks / Trade-offs

- `count_factions` O(N) 在回放中每帧执行 → 10K 单位约 0.1ms，回放非 hot path，可接受
- `HudInteractive` Update 系统每帧执行 → O(1) entity 数，开销可忽略
- `update_top_bar` 回放中运行 → 需访问 `SimulationWorld`，与 `replay_seek_system` 同 schedule → Bevy 自动序列化，安全

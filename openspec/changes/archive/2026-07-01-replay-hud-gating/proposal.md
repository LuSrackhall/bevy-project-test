## Why

回放模式下 HUD 命令下发按钮仍可响应点击注入命令到 `CommandBuffer`，破坏回放确定性。同时回放中阵营兵力信息缺失，HUD 顶部栏显示冻结的虚假初始值，用户体验差。

## What Changes

- **新增** `simulation::world_stats` 模块：纯只读 `count_factions` 函数，`BTreeMap<Faction, (u32,u32)>` 支持动态阵营
- **新增** `HudInteractive` marker + `hide_interactive_in_replay` Update 系统：回放时隐藏 toolbar 和索敌面板
- **修改** HUD observer（3 处）：加 `GameMode::Replay` 提前返回，阻止命令注入
- **修改** HUD Update 系统 gate：除 `update_top_bar` 外全部加 `not(GameMode::Replay)` 闸门
- **修改** `update_top_bar`：改用 `count_factions` 实现动态阵营显示（中文标签），回放中继续运行

## Capabilities

### New Capabilities

- `world-stats`: simulation 层可复用的阵营兵力统计函数，`BTreeMap<Faction, (u32,u32)>` 支持动态阵营，对标 `golden_test::hash_world_state`
- `replay-hud-gating`: 回放模式下 HUD 命令按钮不可交互、工具栏/索敌面板隐藏、顶部状态栏实时更新阵营兵力

### Modified Capabilities

<!-- 无 spec 级需求变更 —— 现有 capability 的 spec 不变 -->

## Impact

- **simulation**: 新增 `world_stats.rs` 模块 + `lib.rs` 注册
- **render_view**: 修改 `hud.rs`（observer gate + `HudInteractive` marker + `count_factions` 调用）、`ui/mod.rs`（系统 gate + 新系统注册）
- **性能**: `count_factions` O(N) 仅在回放中调用；`hide_interactive_in_replay` O(1)；Live 模式零影响

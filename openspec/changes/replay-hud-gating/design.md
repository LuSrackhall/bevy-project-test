## Context

回放模式基于 `GameMode::Replay` 资源标识。输入系统已有 `not(GameMode::Replay)` gate，但 HUD observer 和 Update 系统未完整门控。`setup_hud` 在 `OnEnter(Playing)` 无条件执行（无 replay gate）。

当前代码中 observer 回调通过 `.observe()` 注册，独立于系统 `run_if` 条件触发。

## Goals / Non-Goals

同 brainstorm-spec.md。

## Decisions

### D1: Observer GameMode 检查位置

在每个需要门控的 observer 回调体**首行**加 `if *game_mode == GameMode::Replay { return; }`。`Res<bevy_adapter::GameMode>` 作为闭包参数新增。

三处 observer：
1. `hud.rs` spawn type 按钮（`SpawnTypeBtn`）
2. `hud.rs` toolbar 按钮（`ToolbarButton`）—— 覆盖 shield/force-move
3. `hud.rs` 索敌下发按钮（`SeekIssueBtn`）

理由：observer 不经过 system run condition，只能内联检查。

### D2: Toolbar/索敌隐藏方式

不改 `setup_hud` 签名。用 `HudInteractive` marker + 独立 Update 系统：

```rust
#[derive(Component)]
struct HudInteractive;

fn hide_interactive_in_replay(
    mode: Res<bevy_adapter::GameMode>,
    mut q: Query<&mut Visibility, With<HudInteractive>>,
) {
    let vis = if *mode == bevy_adapter::GameMode::Replay { Visibility::Hidden } else { Visibility::Inherited };
    for mut v in q.iter_mut() { *v = vis; }
}
```

Marker 加在两处：toolbar 容器 Node（hud.rs ~line 322）、索敌面板根节点（`SeekPanelRoot`）。

理由：`OnEnter` 中 `commands.insert_resource` 刷新时序不确定 → 不在 `setup_hud` 中读 `GameMode`。Update 系统在第一帧执行时 `GameMode` 已确定存在。

### D3: world_stats 模块结构

```rust
// simulation/src/world_stats.rs
pub struct FactionCounts {
    pub factions: BTreeMap<Faction, (u32, u32)>, // (soldiers, cities)
}
pub fn count_factions(world: &mut World) -> FactionCounts;
```

两个 query：`(&FactionComponent, &SoldierMarker)` 计兵、`(&FactionComponent, &CityMarker)` 计城。`BTreeMap` 确保确定性迭代。

### D4: HUD Update 闸门拆分

当前 `ui/mod.rs` HUD Update 系统统一用同一个 `run_if`。需拆分：

- `update_top_bar`：**不加** gate，回放中也运行（用于阵营统计显示）
- 其余 9 个系统：加 `not(GameMode::Replay)` gate（性能优化）

实现：`update_top_bar` 移出原 `.add_systems()` 块，单独注册不带 gate。

### D5: update_top_bar 改用 count_factions

将当前的手动 faction query 循环替换为 `simulation::world_stats::count_factions(&mut sim.0)`，用 `counts.factions.iter()` 动态生成显示文本。

## Risks / Trade-offs

- [单帧闪烁] Toolbar/索敌 spawn 后可见、Update 系统才隐藏 → 渲染提取在 Update 之后，实际不可见。风险极低。
- [count_factions 性能] O(N) 回放每帧 → ~0.1ms at 10K units，可接受。不进入 live 热路径。
- [NonSendMut 同 schedule] `update_top_bar` 和 `replay_seek_system` 都访问 `SimulationWorld` → Bevy 自动序列化，安全。

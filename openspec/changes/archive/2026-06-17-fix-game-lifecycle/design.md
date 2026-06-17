## Context

当前 `GameState` 是一个 4 变体的 Bevy `States` 枚举（`MainMenu`/`Playing`/`Paused`/`GameOver`），`SimulationWorld` 在 `main.rs` 启动时创建，`tick_driver_system` 无 `run_if` 守卫。这导致仿真从 app 启动就开始运行，且游戏结束后无法重置。

所有变更集中在 `crates/render_view/`、`crates/bevy_adapter/` 和 `src/main.rs`。`crates/simulation/` 不需要修改（仿真层本身是纯逻辑，问题在适配层和视图层的生命周期管理）。

## Goals / Non-Goal

**Goals:**
- 仿真只在 `GameState::Playing` 时 tick
- 每次进入 Playing 状态时可选择性重置仿真世界（通过 `NeedsGameReset` 标志）
- 暂停不触发任何 `OnEnter`/`OnExit` 钩子
- 所有仿真相关系统都有正确的 `run_if` 守卫

**Non-Goals:**
- 不改变 `simulation` crate 的任何代码
- 不改变暂停的语义（暂停 = 一切停止）
- 不优化暂停/继续的 HUD 重建（当前方案每次暂停后恢复需要重建 HUD）

## Decisions

### 架构：3 状态枚举 + 2 资源

```
GameState:  MainMenu ──→ Playing ──→ GameOver
                 ↑          │  ↑         │
                 └──────────┘  └─────────┘

Paused(bool):          位于 bevy_adapter，tick 守卫直接检查
NeedsGameReset(bool):  位于 render_view，按钮设 true，reset 系统消费后设 false
```

Paused 资源放在 bevy_adapter 而非 render_view，因为 tick_driver_system 的 `run_if` 守卫需要直接访问它。这保持了单向依赖拓扑：bevy_adapter 不需要知道 render_view。

### 系统守卫矩阵

| 系统 | MainMenu | Playing (unpaused) | Playing (paused) | GameOver |
|------|----------|-------------------|------------------|----------|
| tick_driver | OFF | ON | OFF | OFF |
| sync_entities | OFF | ON | OFF | OFF |
| check_victory | OFF | ON | OFF | OFF |
| HUD 更新 | OFF | ON | OFF | OFF |
| 选择/指令 | OFF | ON | OFF | OFF |
| 相机/缩放 | ON | ON | ON | ON |
| 插值 | ON | ON | ON | ON |
| 调试图形 | OFF | ON | OFF | OFF |
| button_style | ON | ON | ON | ON |

### reset_game_system 执行流程

```
OnEnter(Playing):
  1. paused.0 = false
  2. if needs_reset.0:
     a. for e in game_entities: despawn
     b. mapper.clear()
     c. tick_clock = default
     d. cmd_buf.0.clear()
     e. pending.events.clear()
     f. selection.clear()
     g. seed = SystemTime::now().as_secs()
     h. sim_world.0 = init_simulation_world(seed)
     i. generate_map(&mut sim_world.0)
     j. needs_reset.0 = false
  3. setup_hud (after reset, via .after())
```

### 暂停 UI 实现

暂停菜单在 `setup_hud` 中创建（初始 `Visibility::Hidden`），通过 `update_pause_visibility` 系统根据 `Paused` 资源切换可见性。随 HUD 一起在 `OnExit(Playing)` 时销毁。

### Esc 键行为

Esc 无条件设 `Paused(true)`。不先清除选区、不先关闭输入框聚焦。选区清除由点击空地处理，输入框聚焦跨暂停保持。

### 按钮行为变更摘要

| 按钮 | 旧行为 | 新行为 |
|------|--------|--------|
| "单人模式" | `→ Playing` | `NeedsGameReset(true) + → Playing` |
| 暂停"继续" | `→ Playing` | `Paused(false)` |
| 暂停"重新开始" | `→ Playing`（假重启） | `Paused(false) + NeedsGameReset(true) + → MainMenu` |
| 暂停"主菜单" | `→ MainMenu` | `Paused(false) + → MainMenu` |
| 结束"再来一局" | `→ Playing`（假重开） | `NeedsGameReset(true) + → MainMenu` |
| 结束"主菜单" | `→ MainMenu` | `→ MainMenu`（不变） |

## Risks / Trade-offs

- **[权衡] "再来一局"需两步** → 先回 MainMenu 再点"单人模式"。后续可优化为 GameOver 按钮直接设 NeedsGameReset 并跳到 Playing。
- **[风险] 暂停恢复时 HUD 重建** → OnExit(Playing) 销毁 HUD，Paused→Playing 时 OnEnter(Playing) 重建。有短暂视觉闪烁。后续可优化为 HUD 不随暂停销毁。
- **[风险] 暂停期间 GameOver 不触发** → check_victory 在暂停时禁用。继续后才检测。可接受行为。

# 游戏生命周期修复设计

## Context

当前游戏存在两个阻断性 bug：

1. **游戏在点击按钮前就已开始**：`SimulationWorld` 在 `main.rs` 启动时通过 `init_sim_world()` 创建，`tick_driver_system` 在 `Update` 中无条件运行（无 `run_if` 守卫）。仿真从 app 启动那一刻就在跑，主菜单只是视觉层，如果玩家在菜单停留 30 分钟再点击"单人模式"，进入的就是一个已经运行 30 分钟的游戏。

2. **结束后无法重开**：GameOver 的"再来一局"和暂停的"重新开始"按钮只切换 `GameState` 为 `Playing`，但 `SimulationWorld`、`TickClock`、`UnitIdMapper`、`SelectionState`、`CommandBuffer` 从未重置。`check_victory_system` 立即检测到上一局的残留状态，再次触发 GameOver（死循环）。

**根因**：仿真生命周期与 UI 状态混在同一个 `GameState` 枚举中，且缺乏生命周期钩子（`OnEnter`/`OnExit`）来管理仿真世界的初始化和销毁。

## Goals / Non-Goals

**Goals:**
- 仿真只在用户明确开始游戏后才运行
- 支持完整的游戏重开（新随机种子、新地图、全新状态）
- 暂停/继续不破坏游戏状态
- 系统守卫覆盖所有仿真相关系统（tick、sync、选择、指令、HUD 更新、调试图形）
- 符合 CLAUDE.md 宪法要求：仿真纯度、确定性、单向依赖

**Non-Goals:**
- 不改变暂停的语义（暂停 = 一切停止，包括仿真 tick）
- 不引入多人联机、Lockstep 网络、Replay 录像的实现（但架构需兼容）
- 不改变 UI 布局或视觉效果
- 不重构 pause 为 Playing 的子状态（后续优化）

## Decisions

### 决策 1：将 `Paused` 从状态枚举移出，改为布尔资源

**选择**：`GameState` 仅保留 3 个变体（`MainMenu`、`Playing`、`GameOver`），暂停通过独立的 `Paused(bool)` 资源控制。

**理由**：
- Bevy 的 `States` 系统中，`OnExit(Playing)` 在 `Playing → Paused` 转换时会触发（因为它们是同一个枚举的不同变体），导致暂停时 HUD 被意外销毁
- 使用布尔资源后，暂停/继续只是翻转布尔值，不触发任何 `OnEnter`/`OnExit`，避免了状态机碰撞
- 曾考虑过双状态机（`SimulationActive` + `GamePhase`），但会产生无效组合（`Active + Menu`），且需要复杂的组合守卫

**替代方案**：
- Bevy `SubStates`（Paused 作为 Playing 的子状态）：Bevy 0.18 支持不确定
- 双状态机（`SimulationActive` + `GamePhase`）：会产生无效状态组合，增加协调复杂度

### 决策 2：引入 `NeedsGameReset` 标志区分首次开始和暂停恢复

**选择**：添加 `NeedsGameReset(bool)` 资源。`OnEnter(Playing)` 时检查该标志，为 `true` 则执行完整重置，为 `false` 则跳过（暂停恢复）。

**理由**：
- `OnEnter(Playing)` 在每次进入 Playing 状态时都会触发（包括从暂停恢复），但只有首次开始和重开游戏时才需要重置仿真
- 标志由按钮处理器在触发状态转换前设置，`reset_game_system` 在执行后清除

### 决策 3：`Paused` 状态下仿真不 tick

**选择**：`tick_driver_system` 仅在 `GameState::Playing` 且 `Paused(false)` 时运行。

**理由**：
- 符合玩家直觉：暂停 = 一切停止
- 避免了暂停期间的"phantom ticks"问题（无需插入 No-Op 命令）
- `check_victory_system` 同样在暂停时禁用，避免暂停期间弹出 GameOver
- 如果未来需要"后台仿真"（如 AI 持续思考），可添加独立的 `SimulationPaused` 资源

### 决策 4：重开游戏通过 MainMenu 中转

**选择**：GameOver "再来一局"和暂停"重新开始"都先回到 MainMenu，用户再点击"单人模式"开始新游戏。

**理由**：
- 确保 `OnEnter(Playing)` 被触发，执行完整重置流程
- 语义清晰：MainMenu 是游戏的"干净起点"
- UX 代价是多一次点击，但保证了状态转换的正确性

### 决策 5：生命周期钩子职责划分

| 钩子 | 职责 |
|------|------|
| `OnEnter(Playing)` | `reset_game_system`：若 `NeedsGameReset=true`，销毁旧实体、清空状态、新建仿真世界；设 `Paused(false)` |
| `OnExit(Playing)` | `cleanup_playing`：销毁 HUD + 暂停菜单 UI |
| `OnEnter(MainMenu)` | `setup_main_menu`：显示主菜单 |
| `OnExit(MainMenu)` | `cleanup_main_menu`：销毁主菜单 |
| `OnEnter(GameOver)` | `setup_gameover`：显示结束界面 |
| `OnExit(GameOver)` | `cleanup_gameover`：销毁结束界面 |

**理由**：HUD 清理放在 `OnExit(Playing)` 而非 `OnEnter` 目标状态，是因为 Playing→Paused 时不触发 `OnExit(Playing)` 的假设已被证伪——实际上会触发。但由于 `Paused` 已从状态枚举移出，`Playing` 只会转换到 `MainMenu` 或 `GameOver`，`OnExit(Playing)` 不再有歧义。

### 决策 6：暂停 UI 由资源驱动而非状态驱动

**选择**：暂停菜单在 `setup_hud` 中创建（初始 Hidden），通过 `Paused` 资源切换可见性。

**理由**：
- 避免了 `OnEnter(Paused)`/`OnExit(Paused)` 的状态转换副作用
- 暂停菜单的生命周期与 HUD 绑定，随 `OnExit(Playing)` 一起销毁

### 决策 7：随机种子策略

**选择**：每次新游戏使用 `SystemTime` 作为种子。

**理由**：
- 只在游戏初始化时使用，不在仿真 tick 中，不影响确定性
- 同一 tick 内的仿真仍然完全确定性可复现

## Risks / Trade-offs

- **[权衡] "再来一局"需两步操作** → 先回到主菜单再开始新游戏。UX 多一次点击，但保证了完整重置的正确性。后续可优化为在 GameOver 按钮中直接设置 `NeedsGameReset` 并跳过 Menu 显示。
- **[权衡] 暂停时仿真停止** → AI 不会在暂停期间继续思考。如果未来需要"后台仿真"功能，可添加独立的 `SimulationPaused` 资源。
- **[风险] 暂停期间 GameOver 不触发** → `check_victory_system` 在暂停时被禁用。如果玩家最后的单位在暂停期间理论上下一 tick 会死亡，继续后才会检测到。这是可接受的行为（玩家暂停是为了休息，不是为了看游戏结束）。
- **[风险] `NeedsGameReset` 状态残留** → `reset_game_system` 在执行后将其设为 `false`，确保不会意外重置。但如果系统执行顺序异常，可能出现遗漏。通过 `.chain()` 保证执行顺序。
- **[风险] `std::time::SystemTime` 用于种子** → 可观测性差（无法复现同一局游戏）。后续可添加种子显示/输入功能。

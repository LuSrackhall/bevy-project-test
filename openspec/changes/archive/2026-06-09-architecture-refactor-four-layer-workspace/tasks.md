## 1. Workspace 基础设施搭建

- [x] 1.1 将根 `Cargo.toml` 改为 `[workspace]` 格式，`members = ["crates/*"]`，设置 `resolver = "2"`
- [x] 1.2 创建 `crates/simulation/Cargo.toml`：依赖 `bevy_ecs`（default-features=false）、`serde`+`derive`、`ron`、`rand`（no_std compatible features）
- [x] 1.3 创建 `crates/bevy_adapter/Cargo.toml`：依赖 `simulation`（path）、`bevy`（完整版）
- [x] 1.4 创建 `crates/presentation/Cargo.toml`：依赖 `simulation`（path）、`bevy_adapter`（path）、`bevy`
- [x] 1.5 创建 `crates/render_view/Cargo.toml`：依赖 `presentation`（path）、`bevy`、`bevy_prototype_lyon`
- [x] 1.6 创建每个 crate 的 `src/lib.rs` 骨架文件
- [x] 1.7 验证 workspace 全量编译：`cargo check --workspace` 通过（即使各 crate 为空）

## 2. content/ 配置文件

- [x] 2.1 创建 `content/units.ron`：包含 militia/infantry/archer/cavalry 全部字段 + 文件头注释说明格式与修改方式
- [x] 2.2 创建 `content/cities.ron`：包含城池参数 + aura 子对象 + 文件头注释
- [x] 2.3 创建 `content/combat.ron`：包含 shield/cavalry/archer_multi_shot/slow_debuff/level_up/fearless 全部子对象 + 文件头注释
- [x] 2.4 创建 `content/map.ron`：包含地图生成参数 + 文件头注释

## 3. simulation crate — 核心类型与命令体系

- [x] 3.1 实现 `types.rs`：`Fixed(i64)` (8bit 精度) + `Add/Sub/Mul/Div/Eq/Ord/Hash` trait + `from_int`/`from_float`/`to_float` 转换 + `length_squared()` + 单元测试
- [x] 3.2 实现 `types.rs`：`FixedVec2 { x: Fixed, y: Fixed }` + 基本运算 + 单元测试
- [x] 3.3 实现 `types.rs`：`UnitId(pub u64)` + `IdGenerator` (自增) + 单元测试
- [x] 3.4 实现 `types.rs`：`Faction` (Player/Enemy/Neutral) + `SoldierType` (Militia/Infantry/Archer/Cavalry) + `SoldierState` (Moving/Fighting) + `ShieldState` (Normal/ShieldUp)
- [x] 3.5 实现 `command.rs`：`Action` 枚举（MoveTo/Attack/ForceMove/ReturnToCity/SetShield/SetSpawnType/NoOp）
- [x] 3.6 实现 `command.rs`：`GameCommand { tick, player_id, action }` + `CommandBuffer(Vec<GameCommand>)`
- [x] 3.7 实现 `types.rs`：`DeterministicRng` 封装 `SmallRng` + `seed_from_u64` + 单元测试验证确定性

## 4. simulation crate — 配置加载

- [x] 4.1 实现 `soldier/config.rs`：`SoldierUnitConfig` 结构体 + `SoldierConfig` (HashMap<SoldierType, SoldierUnitConfig>) + `serde::Deserialize` derive + 从 `content/units.ron` 加载
- [x] 4.2 实现 `city/config.rs`：`AuraConfig` + `CityGlobalConfig` 结构体 + serde derive + 从 `content/cities.ron` 加载
- [x] 4.3 实现 `combat/config.rs`：`ShieldConfig`/`CavalryConfig`/`ArcherMultiShotConfig`/`SlowDebuffConfig`/`LevelUpConfig`/`FearlessConfig`/`CombatGlobalConfig` + serde derive + 从 `content/combat.ron` 加载
- [x] 4.4 实现 `map/config.rs`：`MapGenConfig` 结构体 + serde derive + 从 `content/map.ron` 加载
- [x] 4.5 实现 `lib.rs`：`init_simulation_world(seed, config_path)` 函数 —— 加载所有配置 → 创建 ECS World → 注入配置 Resources + DeterministicRng → 返回 World

## 5. simulation crate — 仿真组件

- [x] 5.1 实现 `soldier/mod.rs`：定义全部士兵组件（`LogicalPosition`/`Movement`/`Health`/`Attack`/`FactionComponent`/`SoldierTypeComponent`/`Level`/`ShieldComponent`/`CityOrigin`/`SlowDebuff`/`FearlessBuff`）
- [x] 5.2 实现 `city/mod.rs`：定义全部城池组件（`CityComponent`/`CityRadius`/`AuraHeal`/`SpawnDirection`）
- [x] 5.3 实现 `combat/mod.rs`：定义 `Arrow` 逻辑组件（`target: UnitId`/`damage`/`from_faction`/`shooter: Option<UnitId>`/`remaining_ticks: u32`/`last_pos: FixedVec2`）
- [x] 5.4 实现仿真层事件类型：`UnitSpawned`/`UnitDestroyed`/`CityCaptured`/`DamageDealt`/`SoldierLeveledUp`

## 6. simulation crate — 仿真系统（核心）

- [x] 6.1 实现 `command.rs` — `consume_commands_system`：从 `CommandBuffer` 取当前 Tick 命令，应用到组件（Movement/Shield/SpawnType等），缺少命令的 player 注入 NoOp
- [x] 6.2 实现 `soldier/mod.rs` — `soldier_movement_system`：读取 `Movement.target`（UnitId→查 LogicalPosition 或 waypoint FixedVec2）→ 计算方向（定点数）→ 更新 `LogicalPosition`，到达阈值内清 target。平方距离替代真实距离
- [x] 6.3 实现 `soldier/mod.rs` — `city_spawn_system`：遍历城池，冷却 Tick 递减 → `spawn_cooldown == 0` 且 `population < max_population` → `population += 1` → 生成士兵实体 → 重置冷却
- [x] 6.4 实现 `soldier/mod.rs` — `city_capture_check_system`：城池 `Health.current == 0` → 易手、重置状态 → 发出 `CityCaptured` 事件
- [x] 6.5 实现 `soldier/mod.rs` — `city_interaction_system`：士兵进入城池范围 → 敌军：造成伤害（`city_damage_per_soldier_ratio * attack`）→ 友军+有 return 命令：治疗/升级/消耗人口
- [x] 6.6 实现 `soldier/mod.rs` — `aura_heal_system`：友方城池光环范围内士兵每 Tick 回血
- [x] 6.7 实现 `soldier/mod.rs` — `slow_debuff_tick_system`：`SlowDebuff.remaining_ticks` 递减，为 0 时移除组件
- [x] 6.8 实现 `soldier/mod.rs` — `fearless_buff_tick_system`：`FearlessBuff.remaining_ticks` 递减，为 0 时移除
- [x] 6.9 实现 `soldier/mod.rs` — `soldier_level_up_system`：处理 `SoldierLeveledUp` 事件，应用属性提升

## 7. simulation crate — 战斗系统

- [x] 7.1 实现 `combat/mod.rs` — `combat_engagement_system`：非强制移动的单位在 `aggression_range` 内搜索最近敌方 → 设置 `Movement.target` → 状态设为 Fighting。骑兵不因战斗覆盖 command_target
- [x] 7.2 实现 `combat/mod.rs` — `melee_attack_system`：近战单位 Fighting 状态 + 冷却完毕 + 目标在攻击范围 → 计算伤害（含骑兵闪避/无畏/吸血）→ 写入 `Health` → 目标死亡时发出 `UnitDestroyed` + 击杀者经验 → `SoldierLeveledUp`
- [x] 7.3 实现 `combat/mod.rs` — `archer_attack_system`：弓兵在攻击范围内有敌方 → 冷却完毕后发射 Arrow 实体（计算预计命中 Tick）→ 多重射击判定
- [x] 7.4 实现 `combat/mod.rs` — `arrow_hit_system`：检查所有 Arrow → `remaining_ticks == 0` 或目标位置与箭矢重合 → 判伤（含举盾拦截/减速叠层/弓兵近战）→ 发出事件/销毁 Arrow
- [x] 7.5 实现 `combat/mod.rs` — `arrow_expire_system`：Arrow 目标已销毁且 `remaining_ticks` 耗尽 → 销毁

## 8. simulation crate — AI 与地图生成

- [x] 8.1 实现 `map/mod.rs` — `generate_map_system`：使用 `DeterministicRng` + `MapGenConfig` → 生成城池位置（保证最小间距）→ 分配势力 → 发出 `UnitSpawned` 事件（每个城池）
- [x] 8.2 实现 `ai/mod.rs` — `ai_decide_system`：固定评估间隔（配置化 Tick 数）→ 扫描局势 → 向 `CommandBuffer` 写入命令（扩张到中立城/攻击玩家城/回城治疗/升级）。AI 不直接操作组件
- [x] 8.3 实现 `ai/mod.rs` — `ai_defense_system`：低血量城池 → 切换高级兵种 + 召回附近兵力回防

## 9. simulation crate — Tick 调度封装

- [x] 9.1 实现 `lib.rs` — `run_tick(world: &mut World, tick_number: u32, commands: &[GameCommand])`：按阶段顺序依次执行所有系统，返回仿真事件列表。接受命令快照
- [x] 9.2 实现 `lib.rs` — `extract_events(world: &World) -> SimulationEvents`：提取本 Tick 产生的所有事件（供 bevy_adapter 消费）
- [x] 9.3 为 simulation crate 添加独立单元测试：`cargo test -p simulation` 覆盖 Fixed 运算/命令消费/移动系统/战斗系统/AI 决策

## 10. bevy_adapter crate — 映射与 Tick 驱动

- [x] 10.1 实现 `mapper.rs`：`UnitIdMapper { unit_to_entity: HashMap<UnitId, Entity>, entity_to_unit: HashMap<Entity, UnitId> }` + `register()`/`unregister()`/`entity_of()`/`unit_id_of()` 方法
- [x] 10.2 实现 `tick.rs`：`TickClock` 资源（`current_tick`/`tick_duration`/`accumulator`）+ `tick_driver` 系统（每帧累加 delta，满 50ms 执行 Tick）
- [x] 10.3 实现 `tick.rs`：Tick 驱动逻辑 —— 从 `CommandBuffer` 取当前 Tick 命令 → 调用 `simulation::run_tick` → 处理返回事件 → 累积 accumulator 减法

## 11. bevy_adapter crate — 生命周期与输入翻译

- [x] 11.1 实现 `lifecycle.rs` — `sync_spawn_system`：消费 `UnitSpawned` 事件 → `commands.spawn((LogicEntityRef(unit_id),))` → 注册到 `UnitIdMapper`。地图生成的城池同步创建（触发 camera 定位）
- [x] 11.2 实现 `lifecycle.rs` — `sync_destroy_system`：消费 `UnitDestroyed` 事件 → 通过 mapper 查 Entity → `despawn()` → 从 mapper 移除
- [x] 11.3 实现 `lifecycle.rs` — `sync_city_captured_system`：消费 `CityCaptured` 事件 → 不需要额外同步（simulation 已处理所有逻辑），但可用于 HUD 刷新触发
- [x] 11.4 实现 `input.rs` — 右键点击翻译系统（优先级：敌方士兵 > 敌城/中立城 > 友方城 > 空地 waypoint）→ 对每个 selected_unit_id 产生对应 `Action` → 推入 `CommandBuffer`
- [x] 11.5 实现 `input.rs` — Shift/强制移动按钮 → `Action::ForceMove` 翻译
- [x] 11.6 实现 `input.rs` — HUD 按钮翻译（兵种切换 → `Action::SetSpawnType`，举盾 → `Action::SetShield`，强制移动按钮设置 `ForceMoveNext` 标记）

## 12. bevy_adapter crate — 插件组装

- [x] 12.1 实现 `lib.rs` — `BevyAdapterPlugin`：添加 `TickClock`/`CommandBuffer`/`UnitIdMapper`/`ForceMoveNext` 资源，注册所有 tick/lifecycle/input 系统。系统集应在不同 schedule 中：
  - `Update`：tick_driver、input 翻译系统
  - Simulation 执行后的 `Update`：lifecycle 同步系统

## 13. presentation crate — 插值与绑定

- [x] 13.1 实现 `binding.rs`：`LogicEntityRef(pub UnitId)` 组件定义
- [x] 13.2 实现 `binding.rs` — `bind_new_entities_system`：监听 `Added<LogicEntityRef>` → 查询 simulation 层当前 `LogicalPosition` → 转换为浮点 `Vec2` → 插入 `PresentationPosition` + `InterpolationData { previous = current = position, is_new = true }`
- [x] 13.3 实现 `interpolation.rs`：`PresentationPosition(pub Vec2)` + `InterpolationData { previous_logical_pos, current_logical_pos, is_new }` + `RenderInterpolationAlpha(pub f32)` 定义
- [x] 13.4 实现 `interpolation.rs` — `update_interpolation_history_system`：每个 Tick 结束后，将 `current_logical_pos` 挪到 `previous_logical_pos`，从 simulation 读取新的 `LogicalPosition` 写入 `current_logical_pos`，`is_new = false`
- [x] 13.5 实现 `interpolation.rs` — `interpolate_positions_system`：每帧根据 `RenderInterpolationAlpha` 对 `previous_logical_pos` 和 `current_logical_pos` 做线性插值 → 写入 `PresentationPosition`。`is_new == true` 的实体跳过插值，直接使用逻辑位置
- [x] 13.6 实现 `interpolation.rs` — `compute_alpha_system`：每帧从 `TickClock` 读取 `accumulator / tick_duration` → 写入 `RenderInterpolationAlpha`

## 14. presentation crate — 插件组装

- [x] 14.1 实现 `lib.rs` — `PresentationPlugin`：注册 `RenderInterpolationAlpha` 资源，注册 bind/interpolation 系统。系统 schedule：`Update`（compute_alpha → interpolate_positions）、Tick 后 hook（update_interpolation_history）

## 15. render_view crate — DebugShape 渲染

- [x] 15.1 实现 `debug_shape.rs` — `draw_soldiers_system`：Gizmos 读取 `(LogicEntityRef, PresentationPosition, FactionComponent, SoldierTypeComponent)` → 画圆（颜色按阵营、半径按兵种）
- [x] 15.2 实现 `debug_shape.rs` — `draw_cities_system`：Gizmos 读取 `(LogicEntityRef, PresentationPosition, CityRadius)` → 画圆 + 可选绘制光环范围虚线
- [x] 15.3 实现 `debug_shape.rs` — `draw_arrows_system`：Gizmos 读取 `(LogicEntityRef, PresentationPosition)` 箭矢实体 → 画短线段或小圆

## 16. render_view crate — 相机

- [x] 16.1 实现 `camera.rs`：将现有 `src/camera/mod.rs` 迁移到 `render_view/src/camera.rs`，`MainCamera` 标记组件 + `setup_camera` (Camera2d spawn) + `camera_drag_system`（中键/右键拖拽）+ `camera_zoom_system`（滚轮缩放）
- [x] 16.2 实现 `camera.rs` — 初始定位：监听第一个 `Faction::Player` 城池的 `PresentationPosition`，将相机中心移动到该位置
- [x] 16.3 实现 `camera.rs` — 边界限制：相机平移不超过地图 `(0,0) ~ (map_width, map_height)` 范围

## 17. render_view crate — 选择系统

- [x] 17.1 实现 `selection.rs`：将现有 `src/input/mod.rs` 中的选择逻辑迁移到 `render_view/src/selection.rs`
  - `SelectionState` 资源（`selected_unit_ids: Vec<UnitId>`/`selection_mode`/`drag_start`/`drag_current`/`is_dragging`）
  - `SelectionMode` 枚举（Circle/Rect）
  - `SelectionIndicator` 组件
  - `Waypoint` 组件
- [x] 17.2 实现 `selection.rs` — `selection_click_system`：左键点击友方单位 → 替换选区（Ctrl+点击 → 追加）
- [x] 17.3 实现 `selection.rs` — `drag_select_system`：拖拽 → 根据 SelectionMode 做矩形/圆形选择
- [x] 17.4 实现 `selection.rs` — `selection_shortcut_system`：Ctrl+A 全选，Esc 清除选区（若选区非空；否则交由游戏层处理暂停）
- [x] 17.5 实现 `selection.rs` — `selection_visual_system`：被选中士兵显示绿色圆圈指示器
- [x] 17.6 实现 `selection.rs` — `drag_visual_system`：拖拽过程中显示半透明矩形/圆形
- [x] 17.7 实现 `selection.rs` — `waypoint_cleanup_system`：无士兵指向的 waypoint 实体自动销毁

## 18. render_view crate — UI 系统

- [x] 18.1 实现 `ui/mod.rs`：`UiPlugin` 组装 MainMenuPlugin + HudPlugin + PauseMenuPlugin + GameOverPlugin
- [x] 18.2 将现有 `src/ui/menu.rs` 迁移到 `render_view/src/ui/menu.rs`：主菜单 + "单人模式"按钮。点击时发出 `GameCommand` 设置 GameState
- [x] 18.3 将现有 `src/ui/pause.rs` 迁移到 `render_view/src/ui/pause.rs`：暂停菜单 + 继续/重新开始/返回主菜单按钮
- [x] 18.4 将现有 `src/ui/gameover.rs` 迁移到 `render_view/src/ui/gameover.rs`：结算画面 + 胜利/失败文本 + 统计 + 再来一局/返回主菜单
- [x] 18.5 将现有 `src/ui/hud.rs` 迁移到 `render_view/src/ui/hud.rs`，重构为命令驱动：
  - 顶部栏（城池数/人口/时间/暂停按钮）
  - 底部城池面板（等级/HP条/人口/经验）—— 通过 `LogicEntityRef` + `UnitIdMapper` 读取城池数据
  - 底部面板兵种按钮 → 发出 `Action::SetSpawnType` 命令（而非直接修改 City 组件）
  - 底部工具栏（圈选/框选切换/举盾/强制移动）→ 发出对应命令
- [x] 18.6 实现 `ui/hud.rs` — `ForceMoveNext` 资源：强制移动按钮设置 flag，输入系统在下次右键时读取并发出 `Action::ForceMove`

## 19. render_view crate — 游戏状态管理

- [x] 19.1 实现 `GameStats` 资源（start_time/total_kills/player_cities_remaining/winner）+ `check_victory_system` + `handle_pause_input` + OnEnter/OnExit 生命周期系统。从现有 `src/game/mod.rs` 迁移逻辑
- [x] 19.2 实现 `GameState` 状态机：MainMenu → Playing ↔ Paused → GameOver → (Restart)→ Playing / (MainMenu)→ MainMenu

## 20. 组装与迁移 — main.rs 与集成

- [x] 20.1 重写 `src/main.rs`：`App::new()` 添加 `BevyAdapterPlugin` + `PresentationPlugin` + `RenderViewPlugin`，移除所有旧 plugin 引用
- [x] 20.2 在 `main.rs` 中初始化 simulation World（`simulation::init_simulation_world(seed, content_path)`），将 World 作为资源注入 Bevy App（或由 bevy_adapter 管理其生命周期）
- [x] 20.3 确保 bevy_adapter 可以访问 simulation World：通过 `ResMut<SimulationWorld>` 包装或直接 `NonSend<World>`（因为 World 不是 Send）。选择方案：使用 `&mut World` 的 NonSend 包装或自定义资源

## 21. 验证与清理

- [x] 21.1 `cargo check --workspace` 全部通过，无编译错误
- [x] 21.2 `cargo test -p simulation` 全部通过（定点数/命令/系统单元测试）
- [ ] 21.3 `cargo run` 启动游戏进入主菜单 → 单人模式 → 城池生成 → 产兵 → 框选移动 → 战斗 → AI 行为 → 暂停/结算 全部可用
- [ ] 21.4 验证定点数精度：士兵移动 1000 像素后位置与预期偏差 < 1 像素（Fixed 精度误差）
- [ ] 21.5 验证确定性：相同种子 + 相同命令序列运行两次 → 最终 state 完全一致
- [x] 21.6 删除旧 `src/` 下已迁移的模块文件（`core/`、`map/`、`city/`、`soldier/`、`combat/`、`ai/`、`camera/`、`input/`、`ui/`、`game/`）
- [x] 21.7 清理旧 Cargo.toml 中不再需要的依赖（game crate 消除后，检查 bevy/bevy_prototype_lyon/rand 的引用位置）

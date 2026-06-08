## Context

当前项目 `city-conquest` 是一个 Bevy 0.18 单 crate RTS 游戏原型，实现了城池占领、士兵战斗（近战/弓兵/骑兵）、AI 决策、框选控制、HUD 等核心玩法。所有代码约 2000 行分布在 `src/` 下 11 个功能模块中，架构为单层平面结构。

CLAUDE.md 宪法要求建立严格的四层分离架构（simulation → bevy_adapter → presentation → render_view），使用定点数、命令驱动、固定 Tick、确定性仿真。当前代码在以下维度完全不符合宪法：

- **无分层**：仿真逻辑（移动/战斗/光环）与渲染（Shape/Transform）混在同一组件上
- **浮点数泛滥**：所有位置/速度/伤害/范围使用 `f32`/`Vec2`
- **输入直读**：系统内 `Res<ButtonInput<MouseButton>>` 直接驱动游戏状态
- **帧率依赖**：`time.delta_secs()` 用于移动和计时，无固定 Tick
- **Entity 作为业务 ID**：`Soldier.target: Option<Entity>` 将 Bevy 内部句柄暴露为逻辑引用
- **硬编码数值**：所有平衡参数写死在 `core/mod.rs`
- **非确定性随机**：`rand::thread_rng()` 到处使用

本次重构是一次从头重建架构级变更，目标是建立完全符合宪法的代码基础。

## Goals / Non-Goals

**Goals:**
- 建立 4 crate workspace 单行依赖拓扑，由 Cargo 编译器强制依赖方向
- simulation crate 完全隔离，可脱离 Bevy 运行 `cargo test`
- 全定点数仿真，`Fixed(i64)` / `FixedVec2` 替代所有 `f32` / `Vec2`
- 固定 20Hz Tick 命令驱动仿真，兼容未来 Lockstep/Replay
- UnitId + UnitIdMapper 替代 Bevy Entity 作为跨层逻辑标识
- 所有平衡参数外置到 `content/*.ron`，带注释说明
- 迁移所有现有游戏功能到新架构，行为尽量一致但允许改进
- 渲染层可插拔，当前使用 DebugShape（Gizmos），后续可无缝替换正式美术

**Non-Goals:**
- 不新增游戏功能（不引入新兵种、新建筑、新 UI 等）
- 不实现锁步网络同步（架构为此做准备，但本次不实现网络层）
- 不实现录像回放存储格式（架构为此做准备，但本次不实现 I/O 层）
- 不替换正式美术资产（保持在 DebugShape 阶段）
- 不优化性能（不引入多线程、SIMD、LOD 等）

## Decisions

### 决策 1: 定点数精度位选 8 位

`Fixed(i64)` 使用低 8 位作为小数位（精度 1/256 ≈ 0.0039）。选择 8 位的理由：

- 地图 2000×2000 像素，8 位精度下可表示 2^31/256 ≈ 8,388,608 像素范围，远超需求
- 乘法 `(a*b)>>8` 在 i128 中间类型中不会溢出（i64 平方 < i128::MAX）
- 8 位移位在 CPU 上为单指令，无性能损耗
- 16 位精度过高（1/65536），用于 2000 像素地图属于浪费

### 决策 2: Tick 频率 20Hz

- 50ms Tick 间隔是人类操作延迟与仿真精度的平衡点
- 大多数 RTS 游戏使用 8-30Hz（星际争霸 2 约 22.4Hz，帝国时代 2 约 10Hz）
- 攻速 0.5s = 10 Tick，易于整数建模
- 减速持续 1s = 20 Tick

### 决策 3: ECS 命令粒度

每个 `Action` 变体操作单个 `UnitId`（如 `MoveTo { unit: UnitId, target: FixedVec2 }`）。理由：

- 仿真系统对每条命令做单一、无副作用处理
- 回放文件精确定位每个单位的每个决策
- AI Agent 开发友好：文件小而聚焦，模式一致
- 框选 100 个单位右键移动 → 发出 100 条 `MoveTo` 命令。在 20Hz 下，100 条命令的开销远小于仿真计算本身

### 决策 4: 仿真世界独立 ECS World

`simulation` crate 维护独立的 `bevy_ecs::World`，不挂在 Bevy App 的 `World` 中。`bevy_adapter` 通过 `TickDriver` 在每个 Tick 边界手动调用 `simulation::run_tick(&mut world, tick, commands)`。理由：

- simulation World 不含任何 Bevy 渲染组件，类型级别保证纯度
- 可独立于 Bevy App 创建 simulation World 做单元测试
- 避免两个世界间的 Query 冲突

### 决策 5: 箭矢建模

箭矢在仿真层保留为逻辑实体（`Arrow` 组件），但不产生渲染实体——箭矢的飞行过程在仿真层为离散跳转（每 Tick 计算命中），在渲染层通过 Gizmos 画直线表示。理由：

- 如果箭矢只存在于渲染层，则战斗结果是"渲染驱动"的，违反确定性
- 如果箭矢在仿真层做平滑移动，每条箭矢每 Tick 更新位置和命中检测增加大量计算
- 折中方案：仿真层箭矢为"飞行中弹道"——发射时计算预计命中 Tick，到达该 Tick 时直接判伤。渲染层画线表示弹道轨迹

### 决策 6: 配置热加载

本次不实现热加载。`content/*.ron` 在 simulation crate 初始化时解析一次，存入 `Res<XxxConfig>` 资源。理由：

- 热加载需要文件监控线程，引入平台差异和复杂性
- 当前阶段重启游戏即可更新配置，成本可接受
- 后续可在此基础上加 `notify` crate 实现热加载，不影响架构

### 决策 7: 确定性 PRNG

使用 `rand::rngs::SmallRng` + `SeedableRng::seed_from_u64`。种子在 `GameStart` 时由上层传入（开发阶段使用固定种子如 `42`，未来可从网络/UI 获取）。PRNG 作为 `simulation` 内部资源，所有需要随机数的仿真系统通过 `ResMut<DeterministicRng>` 获取。

### 决策 8: 仿真时间单位

所有持续性状态用 Tick 计数而非秒数。例如 `attack_cooldown: u32`（剩余 Tick 数）而非 `attack_timer: f32`。好处：

- 避免浮点数比较（`cooldown == 0` 而非 `timer.elapsed >= duration`）
- 每 Tick 递减 1，简单且确定
- 配置文件中用 `ticks` 标注，由 simulation 初始化时根据 Tick 频率转换

## Risks / Trade-offs

- **[跨层数据同步复杂度]** 四个 crate 间需要清晰的事件/资源契约。两个 ECS World（simulation + Bevy）间的状态同步可能出现遗漏。 → **缓解**：bevy_adapter 作为唯一桥梁，所有跨层通信通过显式事件（`UnitSpawned`, `UnitDestroyed`, `PositionChanged`），不允许旁路。
- **[定点数开发体验]** `Fixed` 类型的算术运算不如 `f32` 直觉，AI Agent 可能写出错误算式。 → **缓解**：在 `Fixed` 上实现完整的 `Add/Sub/Mul/Div` trait + 完备的 `#[test]`，AI 使用时模式和 `i32` 一样。
- **[插值延迟]** 20Hz Tick + 渲染插值 = 逻辑真相比画面延迟 50-100ms。对当前单人本地游戏无影响，未来多人时才会感受到。 → **缓解**：架构已为此预留，可通过减小 Tick 间隔（如 30Hz）或前向预测缓解，不影响本次设计。
- **[Bug 行为变化]** 重构中可能改变某些现有行为（如帧率相关的移动速度微调）。 → **缓解**：接受。用户明确表示"不满意当前实现"，允许架构性改进。
- **[编译时间]** 从 1 crate 变为 4 crate workspace，初始全量编译时间会略有增加。 → **缓解**：增量编译不变；workspace 允许并行编译 crate。长期看独立 crate 减少编译范围（改 simulation 不需重编 render_view）。

## Architecture Diagram

```
┌──────────────────────────────────────────────┐
│                  src/main.rs                  │
│            App::new() 组装 Plugins            │
└──────────────────────────────────────────────┘
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
    ┌──────────┐ ┌──────────┐ ┌──────────┐
    │bevy_     │ │present-  │ │render_   │
    │adapter   │ │tation    │ │view      │
    │          │ │          │ │          │
    │Tick调度  │▶│插值计算  │▶│Gizmo渲染 │
    │ID映射    │ │实体绑定  │ │UI系统    │
    │输入翻译  │ │生灭监听  │ │相机      │
    │生命周期  │ │          │ │选择系统  │
    └────┬─────┘ └──────────┘ └──────────┘
         │
    ┌────▼─────┐
    │simulation│
    │          │
    │纯逻辑ECS │
    │定点数    │
    │Tick系统  │
    │命令驱动  │
    └──────────┘
         ▲
    ┌────┴─────┐
    │ content/ │
    │ .ron配置 │
    └──────────┘
```

## Module Layout

```
city-conquest/
├── Cargo.toml                          # [workspace] members = ["crates/*"]
├── content/
│   ├── units.ron
│   ├── cities.ron
│   ├── combat.ron
│   └── map.ron
├── crates/
│   ├── simulation/
│   │   ├── Cargo.toml                  # deps: bevy_ecs, serde, ron, rand(PRNG only)
│   │   └── src/
│   │       ├── lib.rs                  # pub mod + init_simulation_world()
│   │       ├── types.rs                # Fixed, FixedVec2, UnitId, Faction, SoldierType
│   │       ├── command.rs              # Action, GameCommand, CommandBuffer
│   │       ├── soldier/
│   │       │   ├── mod.rs              # 士兵组件 + 系统
│   │       │   └── config.rs           # SoldierConfig (from content/units.ron)
│   │       ├── city/
│   │       │   ├── mod.rs              # 城池组件 + 系统
│   │       │   └── config.rs           # CityConfig (from content/cities.ron)
│   │       ├── combat/
│   │       │   ├── mod.rs              # 战斗系统（近战+远程+弹道）
│   │       │   └── config.rs           # CombatConfig (from content/combat.ron)
│   │       ├── map/
│   │       │   ├── mod.rs              # 地图生成（确定性）
│   │       │   └── config.rs           # MapConfig (from content/map.ron)
│   │       └── ai/
│   │           └── mod.rs              # AI 决策（通过命令管道操作）
│   ├── bevy_adapter/
│   │   ├── Cargo.toml                  # deps: simulation, bevy(full)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mapper.rs               # UnitIdMapper (双向 HashMap)
│   │       ├── tick.rs                 # TickClock, tick_driver system
│   │       ├── lifecycle.rs            # 实体生灭同步
│   │       └── input.rs                # 输入 → GameCommand 翻译
│   ├── presentation/
│   │   ├── Cargo.toml                  # deps: simulation, bevy_adapter, bevy
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── interpolation.rs        # InterpolationData, RenderInterpolationAlpha
│   │       └── binding.rs              # LogicEntityRef, 生灭监听→渲染实体
│   └── render_view/
│       ├── Cargo.toml                  # deps: presentation, bevy, bevy_prototype_lyon
│       └── src/
│           ├── lib.rs                  # RenderViewPlugin 总装
│           ├── debug_shape.rs          # Gizmos 几何体渲染
│           ├── camera.rs               # MainCamera + 拖拽/缩放
│           ├── selection.rs            # 框选/点选视觉 + SelectionState
│           └── ui/
│               ├── mod.rs              # UiPlugin
│               ├── hud.rs              # 顶部栏 + 底部面板 + 工具栏
│               ├── menu.rs             # 主菜单
│               ├── pause.rs            # 暂停菜单
│               └── gameover.rs         # 结算画面
```

## Tick Pipeline

每个 50ms Tick 按以下顺序执行仿真阶段（simulation crate 内部）：

```
Tick N:
1. consume_commands     — 取 CommandBuffer 中标为 Tick N 的命令，应用到组件
2. combat_evaluate      — 计算所有攻击（近战+远程），产生 DamageEvent
3. soldier_movement     — 根据 Movement 组件和速度更新 LogicalPosition
4. city_spawn           — 城池产兵（人口+1），冷却 Tick 递减
5. city_capture_check   — 城池 HP ≤ 0 → 易手，重置状态
6. city_interaction     — 士兵抵达城池 → 治疗/升级/攻击
7. aura_heal            — 友方城池光环治疗范围内单位
8. soldier_level_up     — 处理升级事件（经验→等级→属性提升）
9. ai_decide            — AI 向 CommandBuffer 写入 N+1 Tick 命令
10. archive_commands    — 清空已消费命令（保留副本供回放）
```

## Data Flow: 右键移动命令的完整路径

```
1. render_view: 用户右键点击空地
2. render_view: 读取 SelectionState，收集 selected_unit_ids
3. render_view: 读取 camera 视口转换，获得世界坐标
4. render_view: 对每个选中的 UnitId，发出 MoveTo 命令到 CommandBuffer
5. bevy_adapter: tick_driver 检测到 accumulator >= 50ms
6. bevy_adapter: 运行 simulation::run_tick(world, tick, commands)
7. simulation: consume_commands 阶段，对每个 MoveTo 写入 Movement.target
8. simulation: soldier_movement 阶段，读取 Movement.target → 更新 LogicalPosition
9. simulation: 完成所有 Tick 阶段
10. bevy_adapter: 发出 PositionUpdated 事件（含 UnitId + 新 LogicalPosition）
11. presentation: 监听 PositionUpdated → 更新 InterpolationData
12. presentation: interpolate_positions 每帧计算 PresentationPosition
13. render_view: draw_soldiers 读取 PresentationPosition → 画圆
```

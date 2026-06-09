## ADDED Requirements

### Requirement: 定点数类型体系

simulation crate SHALL 提供 `Fixed(i64)` 定点数类型，使用低 8 位作为小数精度位（精度 1/256 ≈ 0.0039）。SHALL 提供 `FixedVec2 { x: Fixed, y: Fixed }` 二维向量类型。SHALL 实现所有必要的算术和比较 trait（`Add, Sub, Mul, Div, Eq, Ord, Hash`）。

#### Scenario: Fixed 基本运算

- **WHEN** `Fixed::from_int(3)` + `Fixed::from_int(5)`
- **THEN** 结果等于 `Fixed::from_int(8)`

#### Scenario: Fixed 乘法精度

- **WHEN** `Fixed::from_int(3)` * `Fixed::from_float(0.5)`
- **THEN** 结果约等于 `Fixed::from_int(1)`，误差 < Fixed(1)

#### Scenario: FixedVec2 平方距离

- **WHEN** `FixedVec2 { x: Fixed::from_int(3), y: Fixed::from_int(4) }.length_squared()`
- **THEN** 结果等于 `Fixed::from_int(25)`

#### Scenario: 禁止真实距离方法

- **WHEN** 审查 `FixedVec2` 的公开 API
- **THEN** SHALL NOT 提供 `length()` 或任何开方计算方法

### Requirement: UnitId 逻辑标识

simulation crate SHALL 定义 `UnitId(pub u64)` 作为所有仿真实体的唯一逻辑标识符。所有仿真组件之间的引用 SHALL 使用 `UnitId`，SHALL NOT 使用 Bevy `Entity`。

#### Scenario: UnitId 唯一性

- **WHEN** `IdGenerator::next()` 被连续调用 3 次
- **THEN** 产生 3 个互不相同的 `UnitId` 值

#### Scenario: UnitId 作为组件间引用

- **WHEN** `Soldier` 组件存储攻击目标
- **THEN** 目标字段类型为 `Option<UnitId>`，而非 `Option<Entity>`

### Requirement: GameCommand 命令体系

simulation crate SHALL 定义基于 `GameCommand { tick, player_id, action }` 的命令模型，其中 `Action` 枚举包含所有可执行操作变体。SHALL 提供 `CommandBuffer` 资源作为命令队列。

#### Scenario: 命令入队

- **WHEN** 向 `CommandBuffer` 压入一条 `GameCommand { tick: 5, player_id: 0, action: MoveTo { unit: ..., target: ... } }`
- **THEN** `CommandBuffer.0` 包含该命令

#### Scenario: 命令在指定 Tick 消费

- **WHEN** 执行 Tick 5 的 `consume_commands` 阶段
- **THEN** 仅消费标记为 `tick: 5` 的命令，Tick 6 的命令保持不变

#### Scenario: No-Op 补齐

- **WHEN** 某个 Tick 没有对应 player 的命令
- **THEN** 系统 SHALL 为该 player 自动注入 `Action::NoOp` 命令，保证每个玩家每个 Tick 至少有一条命令

### Requirement: 固定 Tick 仿真调度

simulation crate SHALL 在 `run_tick(world, tick_number)` 函数中按固定顺序执行仿真阶段。阶段顺序 SHALL 为：consume_commands → combat_evaluate → soldier_movement → city_spawn → city_capture_check → city_interaction → aura_heal → soldier_level_up → ai_decide → archive_commands。SHALL NOT 依赖帧率或系统时钟。

#### Scenario: Tick 顺序确定性

- **WHEN** 以相同世界状态和相同命令输入执行 Tick N 两次
- **THEN** 两次执行后的世界状态完全相同（逐组件逐字段一致）

#### Scenario: 无帧率依赖

- **WHEN** 两次 `run_tick` 调用之间有任意时间间隔
- **THEN** 仿真结果仅取决于传入的 `tick_number` 参数和当前世界状态，不受实际间隔时间影响

### Requirement: 确定性随机数

simulation crate SHALL 使用 `SeedableRng` 特质的确定性 PRNG（`SmallRng` + `seed_from_u64`）。所有需要随机性的仿真系统 SHALL 通过 `ResMut<DeterministicRng>` 获取随机源。SHALL NOT 使用 `rand::thread_rng()` 或任何非确定性随机源。

#### Scenario: 相同种子相同结果

- **WHEN** 使用种子 `42` 创建两个独立的 simulation World，注入完全相同的命令序列执行 100 个 Tick
- **THEN** 两个世界的最终状态逐组件逐字段完全一致

#### Scenario: 地图生成确定性

- **WHEN** 使用相同种子和相同 `MapConfig` 生成地图两次
- **THEN** 城池位置、等级、势力分布完全相同

### Requirement: 仿真层依赖限制

simulation crate 的 `Cargo.toml` SHALL NOT 依赖 `bevy`（完整版）、`bevy_render`、`bevy_ui`、`bevy_window`、`bevy_input`、`bevy_audio`、`bevy_asset` 或任何图形/窗口/音频 crate。SHALL 仅依赖 `bevy_ecs` 核心子集（+ `serde`、`ron`、`rand`）。

#### Scenario: 编译时隔离验证

- **WHEN** 在 `simulation/src/` 中尝试 `use bevy::prelude::Transform`
- **THEN** 编译失败，因为 `bevy` 不在 `simulation` 的依赖中

#### Scenario: 独立运行测试

- **WHEN** 在 `crates/simulation/` 目录下执行 `cargo test`
- **THEN** SHALL 在无 Bevy 完整运行时的情况下成功编译并运行所有测试

### Requirement: 士兵组件与系统

simulation crate SHALL 定义士兵相关组件：`LogicalPosition`、`Movement`、`Health`、`Attack`、`FactionComponent`、`SoldierTypeComponent`、`Level`、`ShieldComponent`、`CityOrigin`、`SlowDebuff`。SHALL 提供 `soldier_movement_system` 基于 `Movement.target`（UnitId 或 waypoint 位置）和 `Movement.speed` 更新 `LogicalPosition`。

#### Scenario: 士兵向目标移动

- **WHEN** 士兵的 `Movement.target = Some(waypoint_id)`，且 waypoint 的 `LogicalPosition` 距离士兵位置 100 像素（Fixed 单位）
- **THEN** 执行一次 Tick 后，士兵的 `LogicalPosition` 向目标方向移动了 `speed * tick_duration` 距离

#### Scenario: 士兵到达目标

- **WHEN** 士兵的 `LogicalPosition` 和目标位置的平方距离 < 阈值平方（如 Fixed::from_int(5)²）
- **THEN** `Movement.target` 被清除为 `None`，士兵停止移动

#### Scenario: 骑兵不受战斗目标覆盖

- **WHEN** 骑兵处于 `Fighting` 状态且 `Movement.command_target` 不为 `None`
- **THEN** 骑兵继续向 `Movement.command_target` 移动，不因战斗状态而停在原地攻击

### Requirement: 城池组件与系统

simulation crate SHALL 定义城池相关组件：`CityComponent`（等级/人口/经验/产兵类型/冷却）、`CityRadius`、`AuraHeal`。SHALL 提供 `city_spawn_system` 按冷却间隔增加人口并生成士兵实体。

#### Scenario: 城池产兵

- **WHEN** 城池 `population < max_population` 且 `spawn_cooldown == 0`
- **THEN** `population += 1`，`spawn_cooldown` 重置为配置的产兵间隔（Tick 数），并在城池半径外加一定距离生成一个新士兵实体

#### Scenario: 中立城池不产兵

- **WHEN** 城池的 `faction == Faction::Neutral`
- **THEN** `city_spawn_system` 跳过该城池，不改变 `population` 和 `spawn_cooldown`

#### Scenario: 城池易手

- **WHEN** 城池的 `health.current == 0`
- **THEN** `faction` 变更为最后攻击者的阵营，`level = max(level-1, 1)`，`health` 重置为 `max_health * capture_hp_ratio`，`population = 0`
- **AND** 触发 `CityCaptured` 事件

### Requirement: 战斗系统

simulation crate SHALL 提供战斗系统，支持近战攻击和远程（弓箭手）攻击。战斗结果 SHALL 仅取决于 Tick 序号、单位状态和 PRNG，不依赖帧率。

#### Scenario: 近战攻击计算

- **WHEN** 近战单位处于 `Fighting` 状态，目标在攻击范围内，且攻击冷却已过
- **THEN** 对目标 `Health.current` 造成 `attack.damage` 点伤害，重置攻击冷却

#### Scenario: 骑兵闪避

- **WHEN** 骑兵单位受到近战伤害
- **THEN** 根据 `cavalry_dodge_chance(health_ratio)` 概率判定是否闪避。闪避成功时不受伤害并激活 FearlessBuff；闪避失败时正常受伤害

#### Scenario: 弓箭手在范围外不攻击

- **WHEN** 弓箭手的 `Attack.range` 内没有敌方单位
- **THEN** 弓箭手不发射箭矢，攻击冷却不重置

#### Scenario: 箭矢飞行与命中

- **WHEN** 弓箭手发射箭矢，指定目标单位和预计命中 Tick
- **THEN** 在预计命中 Tick（或目标的 `LogicalPosition` 与该箭矢直线距离 < 命中阈值时）判伤；如果目标在命中前被销毁，箭矢继续沿惯性方向飞行直至超时销毁

#### Scenario: 弓兵多重射击

- **WHEN** 弓兵攻击冷却完成，且 `archer_multi_shot_chance(level)` 随机判定通过
- **THEN** 向范围内最近 2-5 个目标分别发射一支箭矢

### Requirement: AI 命令驱动决策

simulation crate SHALL 提供 AI 决策系统，AI SHALL 通过向 `CommandBuffer` 写入 `GameCommand` 来控制自己的单位。AI SHALL NOT 直接操作任何单位或城池的组件。AI SHALL 以固定评估间隔（如每 50 Tick）运行。

#### Scenario: AI 通过命令管道操作

- **WHEN** AI 决定命令一个士兵移向敌方城池
- **THEN** AI 向 `CommandBuffer` 写入 `GameCommand { tick: N+1, player_id: 1, action: MoveTo { unit: ..., target: ... } }` 而非直接设置 `Movement.target`

#### Scenario: AI 评估间隔

- **WHEN** 当前 Tick 不是 AI 评估间隔的倍数
- **THEN** AI 决策系统不执行任何操作，不向 `CommandBuffer` 写入新命令

### Requirement: 事件系统

simulation crate SHALL 定义仿真层事件用于跨系统通信和跨层通知：`UnitSpawned`、`UnitDestroyed`、`CityCaptured`、`DamageDealt`、`SoldierLeveledUp`。事件 SHALL 在相应系统阶段内发出。

#### Scenario: 单位销毁事件

- **WHEN** 单位 `Health.current == 0`
- **THEN** combat 系统发出 `UnitDestroyed { unit_id, killer_id, ... }` 事件，随后实体从 simulation World 中移除

#### Scenario: 经验获取与升级事件

- **WHEN** 单位击杀敌方获得经验导致 `exp >= exp_to_level`
- **THEN** `exp -= exp_to_level`，`level += 1`，`max_health += hp_gain`，`attack += attack_gain`，发出 `SoldierLeveledUp` 事件

### Requirement: 配置加载

simulation crate SHALL 在初始化时从 `content/` 目录的 `.ron` 文件加载配置（兵种属性、城池参数、战斗公式、地图参数）。配置 SHALL 被解析为类型安全的 Rust 结构体，并作为 ECS 资源注入 simulation World。

#### Scenario: 兵种配置加载

- **WHEN** `content/units.ron` 中 militia 的 `health` 配置为 `100`
- **THEN** 解析后 `SoldierConfig::militia.health == 100`，且该值在创建 militia 士兵时用于初始化 `Health` 组件

#### Scenario: 配置缺失时的行为

- **WHEN** 配置文件缺失或格式错误
- **THEN** 系统 SHALL panic 并输出明确的错误信息指出缺失的字段和文件路径（开发阶段不静默降级）

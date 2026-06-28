# simulation-crate

## Purpose

TBD
## Requirements
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

simulation crate SHALL 定义基于 `GameCommand { tick, player_id, action }` 的命令模型，其中 `Action` 枚举包含所有可执行操作变体。SHALL 提供 `CommandBuffer` 资源作为命令队列。`Action` SHALL 提供 `fn sort_tag(&self) -> u8` 方法，返回显式硬编码的排序标签。sort_tag 返回值 SHALL 用于同一 Tick 内命令的确定性排序。

#### Scenario: 命令入队

- **WHEN** 调用 `CommandBuffer::push(GameCommand { tick: 1, player_id: 0, action: Action::Stop })`
- **THEN** `CommandBuffer` 中包含该命令

#### Scenario: sort_tag 返回固定值

- **WHEN** 调用 `Action::Stop.sort_tag()`
- **THEN** 返回固定 u8 值，不依赖 Rust 枚举隐式判别值

#### Scenario: sort_tag 排序一致性

- **WHEN** 同一 tick 有多条命令按 (player_id, action.sort_tag()) 排序
- **THEN** 排序结果在不同编译环境下一致

### Requirement: 固定 Tick 仿真调度

run_tick SHALL 实现完整的六步 Tick 时序：(1) 指令收集（take_for_tick）；(2) 指令补齐（缺失玩家注入 NoOp）；(3) 指令排序（player_id, action.sort_tag()）；(4) 指令归档（可选 ReplayFile Resource）；(5) 确定性仿真；(6) 状态输出。

#### Scenario: 空命令 tick 注入 NoOp

- **WHEN** 某 tick 的 CommandBuffer 中无 Player(0) 的命令
- **THEN** run_tick 自动为 Player 注入 Action::NoOp

#### Scenario: 命令按 sort_tag 排序

- **WHEN** 同一 tick 有多条不同 player_id 的命令
- **THEN** 命令在执行前按 (player_id, action.sort_tag()) 排序

#### Scenario: Neutral 不注入 NoOp

- **WHEN** World 中存在 Neutral 阵营的实体
- **THEN** 不为 Neutral 注入 NoOp（仅 Player 和 Enemy）

#### Scenario: 归档可选

- **WHEN** World 中无 ReplayFile Resource
- **THEN** run_tick 跳过归档步骤，正常执行

### Requirement: 确定性随机数

simulation crate SHALL 使用 `SeedableRng` 特质的确定性 PRNG（`SmallRng` + `seed_from_u64`）。所有需要随机性的仿真系统 SHALL 通过 `ResMut<DeterministicRng>` 获取随机源。SHALL NOT 使用 `rand::thread_rng()` 或任何非确定性随机源。

#### Scenario: 相同种子相同结果

- **WHEN** 使用种子 `42` 创建两个独立的 simulation World，注入完全相同的命令序列执行 100 个 Tick
- **THEN** 两个世界的最终状态逐组件逐字段完全一致

#### Scenario: 地图生成确定性

- **WHEN** 使用相同种子和相同 `MapConfig` 生成地图两次
- **THEN** 城池位置、等级、势力分布完全相同

### Requirement: 仿真层依赖限制

simulation crate 的 `Cargo.toml` SHALL NOT 依赖 `bevy`（完整版）、`bevy_render`、`bevy_ui`、`bevy_window`、`bevy_input`、`bevy_audio`、`bevy_asset` 或任何图形/窗口/音频 crate。SHALL 仅依赖 `bevy_ecs` 核心子集（+ `serde`、`ron`、`rand`）。`bevy_ecs` 版本 SHALL 为 `0.19`。

#### Scenario: 编译时隔离验证

- **WHEN** 在 `simulation/src/` 中尝试 `use bevy::prelude::Transform`
- **THEN** 编译失败，因为 `bevy` 不在 `simulation` 的依赖中

#### Scenario: 独立运行测试

- **WHEN** 在 `crates/simulation/` 目录下执行 `cargo test`
- **THEN** SHALL 在无 Bevy 完整运行时的情况下成功编译并运行所有测试

#### Scenario: bevy_ecs 0.19 Resources as Components 兼容

- **WHEN** simulation crate 中的 `#[derive(Resource)]` 类型在 bevy_ecs 0.19 中使用
- **THEN** 资源作为组件存储在专用抽象实体上，不影响仿真层的 `world.resource::<T>()` 和 `world.resource_mut::<T>()` 读写语义

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

### Requirement: DefaultHasher 禁用

simulation crate 的 hash_world_state SHALL NOT 使用 `std::collections::hash_map::DefaultHasher`。SHALL 使用跨 Rust 版本稳定的确定性哈希函数。

#### Scenario: hash 跨版本稳定

- **WHEN** 使用确定性哈希函数对同一 World 状态计算 hash
- **THEN** 结果在不同 Rust 编译器版本下一致

### Requirement: hash_world_state 组件覆盖

hash_world_state SHALL 覆盖所有影响仿真结果的组件和字段。本次新增覆盖：Movement 的 command_target 和 waypoint 字段，CityComponent 的 max_level、spawn_type、last_attacker_faction、arrow_damage_acc 字段，以及 CityOrigin 和 SoldierStateComponent 组件。

#### Scenario: Movement 字段完整覆盖

- **WHEN** 对包含 command_target 和 waypoint 的 Movement 组件计算 hash
- **THEN** 这些字段影响最终 hash 结果

#### Scenario: CityComponent 字段完整覆盖

- **WHEN** 对包含 spawn_type 和 arrow_damage_acc 的 CityComponent 计算 hash
- **THEN** 这些字段影响最终 hash 结果

### Requirement: run_tick 三参数签名

run_tick SHALL 接受 `(world: &mut World, tick_number: u32, config: &RunConfig)` 三个参数。SHALL 提供 `run_tick_default(world, tick)` 兼容包装。

#### Scenario: run_tick 内部条件执行 ai_decide

- **WHEN** config.enable_ai 为 false 时调用 run_tick
- **THEN** ai_decide 阶段跳过执行

#### Scenario: run_tick_default 行为不变

- **WHEN** 调用 run_tick_default(world, tick)
- **THEN** 行为等价于原 run_tick(world, tick)

### Requirement: consume_commands_system 签名变更

consume_commands_system SHALL 接收外部 `Vec<GameCommand>` 参数，SHALL NOT 内部调用 take_for_tick。

#### Scenario: 接收预排序命令

- **WHEN** 调用 consume_commands_system(world, sorted_commands)
- **THEN** 直接遍历 sorted_commands 执行，不再从 CommandBuffer 取命令

### Requirement: gen_probability 移除

DeterministicRng SHALL NOT 包含返回 f32 的 gen_probability() 方法。

#### Scenario: 无 f32 概率方法

- **WHEN** 审查 DeterministicRng 的公开 API
- **THEN** 不存在返回 f32 的方法，仅有 gen_probability_permyriad() 返回 u32

### Requirement: bevy_adapter DefaultHasher 替换

bevy_adapter 中的 world_fingerprint 函数 SHALL NOT 使用 DefaultHasher，SHALL 使用 FNV-1a 或等价的确定性哈希。

#### Scenario: 跨版本稳定

- **WHEN** world_fingerprint 在不同 Rust 编译器版本下运行
- **THEN** 相同输入产生相同哈希值

### Requirement: CI 自动化检查

CI SHALL 包含以下检查步骤：simulation 禁用类型扫描、浮点渗入检测、依赖拓扑检查。

#### Scenario: 禁用类型扫描

- **WHEN** simulation crate 引入 bevy_render 或 bevy_window
- **THEN** CI 立即失败

#### Scenario: 浮点渗入检测

- **WHEN** simulation crate 中出现非白名单的 f32/f64 使用
- **THEN** CI 立即失败

### Requirement: UnitIdEntityIndex

simulation crate SHALL 在每 tick 开头重建 `UnitIdEntityIndex(HashMap<UnitId, Entity>)` Resource，从全量实体推导。SHALL 用于替代所有 `find_entity_by_unit_id` 线性扫描。

#### Scenario: O(1) 查找

- **WHEN** 调用 UnitIdEntityIndex 查找 UnitId 对应的 Entity
- **THEN** 复杂度 O(1)，不遍历全部实体

#### Scenario: 每 tick 重建

- **WHEN** run_tick 执行时
- **THEN** 在 Step 5 确定性仿真之前重建 UnitIdEntityIndex


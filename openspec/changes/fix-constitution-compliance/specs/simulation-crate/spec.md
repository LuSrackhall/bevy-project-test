## MODIFIED Requirements

### Requirement: Tick 时序六步流程

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

### Requirement: consume_commands_system 签名变更

consume_commands_system SHALL 接收外部 `Vec<GameCommand>` 参数，SHALL NOT 内部调用 take_for_tick。

#### Scenario: 接收预排序命令

- **WHEN** 调用 consume_commands_system(world, sorted_commands)
- **THEN** 直接遍历 sorted_commands 执行，不再从 CommandBuffer 取命令

### Requirement: gen_probability 移除

DeterministicRng SHALL NOT 包含返回 f32 的 gen_probability() 方法。

#### Scenario: 无 f32 概率方法

- **WHEN** 审查 DeterministicRng 的公开 API
- **THEN** 不存在返回 f32 的方法，仅有 gen_probability_permyriad() 返回 u16

### Requirement: hash_world_state 字段补齐

hash_world_state SHALL 覆盖 Movement 的 command_target 和 waypoint 字段，CityComponent 的 max_level、spawn_type、last_attacker_faction、arrow_damage_acc 字段，以及 CityOrigin 和 SoldierStateComponent 组件。

#### Scenario: Movement 字段完整覆盖

- **WHEN** 对包含 command_target 和 waypoint 的 Movement 组件计算 hash
- **THEN** 这些字段影响最终 hash 结果

#### Scenario: CityComponent 字段完整覆盖

- **WHEN** 对包含 spawn_type 和 arrow_damage_acc 的 CityComponent 计算 hash
- **THEN** 这些字段影响最终 hash 结果

### Requirement: bevy_adapter DefaultHasher 替换

bevy_adapter 中的 world_fingerprint 函数 SHALL NOT 使用 DefaultHasher，SHALL 使用 FNV-1a 或等价的确定性哈希。

#### Scenario: 跨版本稳定

- **WHEN** world_fingerprint 在不同 Rust 编译器版本下运行
- **THEN** 相同输入产生相同哈希值

### Requirement: CI 自动化检查补齐

CI SHALL 包含以下检查步骤：simulation 禁用类型扫描、浮点渗入检测、hash_world_state 覆盖率检查、依赖拓扑检查。

#### Scenario: 禁用类型扫描

- **WHEN** simulation crate 引入 bevy_render 或 bevy_window
- **THEN** CI 立即失败

#### Scenario: 浮点渗入检测

- **WHEN** simulation crate 中出现非白名单的 f32/f64 使用
- **THEN** CI 立即失败

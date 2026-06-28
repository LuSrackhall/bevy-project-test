## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: DefaultHasher 禁用

simulation crate 的 hash_world_state SHALL NOT 使用 `std::collections::hash_map::DefaultHasher`。SHALL 使用跨 Rust 版本稳定的确定性哈希函数。

#### Scenario: hash 跨版本稳定

- **WHEN** 使用确定性哈希函数对同一 World 状态计算 hash
- **THEN** 结果在不同 Rust 编译器版本下一致

### Requirement: hash_world_state 组件覆盖

hash_world_state SHALL 覆盖所有影响仿真结果的组件，包括但不限于：UnitIdComponent、LogicalPosition、Health、Attack、Movement、FactionComponent、SoldierTypeComponent、Level、CityComponent、ShieldItem、SeekStance、SlowDebuff、FearlessBuff、ShieldComponent、AttackWindup、FacingDirection、Arrow、DroppedShield。

#### Scenario: 新增组件被哈希覆盖

- **WHEN** 对包含 SeekStance、SlowDebuff、FearlessBuff 等组件的 World 计算 hash
- **THEN** 这些组件的字段值影响最终 hash 结果

#### Scenario: hash 覆盖完整性

- **WHEN** 新增仿真组件时
- **THEN** hash_world_state MUST 同步更新覆盖（宪法 §10.2 要求）

### Requirement: run_tick 三参数签名

run_tick SHALL 接受 `(world: &mut World, tick_number: u32, config: &RunConfig)` 三个参数。SHALL 提供 `run_tick_default(world, tick)` 兼容包装。

#### Scenario: run_tick 内部条件执行 ai_decide

- **WHEN** config.enable_ai 为 false 时调用 run_tick
- **THEN** ai_decide 阶段跳过执行

#### Scenario: run_tick_default 行为不变

- **WHEN** 调用 run_tick_default(world, tick)
- **THEN** 行为等价于原 run_tick(world, tick)

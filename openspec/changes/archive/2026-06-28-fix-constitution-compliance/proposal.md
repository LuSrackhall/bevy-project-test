## Why

宪法合规审计发现 5 项违规：§3.1 Tick 时序缺少 No-Op 注入、命令排序、命令归档三个步骤；§2.3 残留 f32 deprecated 方法；§10.2 hash_world_state 组件/字段覆盖不完整；§10.3 bevy_adapter 残留 DefaultHasher；§22 CI 缺少 4 项自动化检查。这些都是宪法 Tier 1 硬约束，违反即不合格。

## What Changes

- **BREAKING** `consume_commands_system` 签名变更：从内部 `take_for_tick` 改为接收外部 `Vec<GameCommand>`。
- **BREAKING** `run_tick` 内部实现完整六步 Tick 时序（收集→补齐→排序→归档→仿真→输出）。
- 新增 `collect_command_players(world)` 函数，显式 match 映射 Faction→player_id。
- `ReplayFile` 加 `derive(Resource)`，作为可选 Resource 支持 run_tick 内部归档。
- `Scenario::run()` 删除自行排序（由 run_tick 内部处理）。
- 删除 `types.rs` 中 `gen_probability()` deprecated f32 方法。
- `bevy_adapter/driver.rs` 的 `world_fingerprint` 中 DefaultHasher 替换为 FNV-1a。
- 补齐 `hash_world_state` 的 Movement（command_target, waypoint）、CityComponent（max_level, spawn_type, last_attacker_faction, arrow_damage_acc）、CityOrigin、SoldierStateComponent 覆盖。
- CI 添加 4 个检查步骤：禁用类型扫描、浮点渗入检测、hash 覆盖率检查、依赖拓扑检查。

## Capabilities

### New Capabilities
（无）

### Modified Capabilities
- `simulation-crate`: run_tick 六步时序、consume_commands_system 签名变更、hash 覆盖补齐、gen_probability 移除
- `run-config`: 无需求变更（ADR 已记录）

## Impact

- **API 破坏**：`consume_commands_system` 签名变更影响 4 个 seek_stance 测试。
- **确定性影响**：排序步骤可能改变多玩家同 tick 命令的执行顺序。NoOp 注入是空操作，不影响仿真状态。
- **ReplayFile derive 变更**：加 Resource marker trait，不影响序列化。
- **CI 增强**：4 个新增检查步骤，全部通过方可合并。

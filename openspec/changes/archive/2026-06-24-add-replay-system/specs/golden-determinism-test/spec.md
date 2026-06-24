## ADDED Requirements

### Requirement: 黄金确定性测试存在

simulation crate SHALL 包含至少 3 个黄金确定性测试用例，在 CI 中每次提交自动运行。

#### Scenario: 空地图无指令测试
- **WHEN** 使用固定 seed 初始化空地图，执行 1000 tick（无外部指令）
- **THEN** 最终世界状态与基线哈希完全一致

#### Scenario: 1v1 战斗测试
- **WHEN** 使用固定 seed 初始化地图，注入预定义的战斗指令序列，执行 500 tick
- **THEN** 最终世界状态与基线哈希完全一致

#### Scenario: 多城市混战测试
- **WHEN** 使用固定 seed 初始化大地图，注入包含 AI 决策、城市争夺、多兵种战斗的指令序列，执行 2000 tick
- **THEN** 最终世界状态与基线哈希完全一致

### Requirement: 世界状态哈希函数

simulation crate SHALL 提供 `hash_world_state(world: &World) -> u64` 函数，对所有实体的所有组件按 UnitId 排序后计算确定性哈希。

#### Scenario: 相同状态相同哈希
- **WHEN** 两个 World 包含完全相同的实体和组件值
- **THEN** `hash_world_state()` 返回相同的 u64 值

#### Scenario: 不同状态不同哈希
- **WHEN** 两个 World 仅有一个实体的 Health 值不同
- **THEN** `hash_world_state()` 返回不同的 u64 值

### Requirement: 黄金测试可在 simulation crate 独立运行

黄金测试 SHALL 可通过 `cargo test -p simulation` 独立运行，不依赖 bevy_adapter 或 render_view。

#### Scenario: 独立测试运行
- **WHEN** 在 simulation crate 目录执行 `cargo test`
- **THEN** 黄金确定性测试全部通过，无需编译其他 crate

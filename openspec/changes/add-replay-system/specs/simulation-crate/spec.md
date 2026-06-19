## MODIFIED Requirements

### Requirement: 定点数类型体系

Fixed(i64) 和 FixedVec2 SHALL 派生 `Serialize` 和 `Deserialize`。Fixed SHALL 序列化为底层 i64（透明序列化），不做人类可读转换。

#### Scenario: Fixed 序列化往返
- **WHEN** 创建 `Fixed(12345)` 并序列化为 RON
- **THEN** 反序列化后值为 `Fixed(12345)`

### Requirement: GameCommand 命令体系

GameCommand、Action 及其所有嵌套类型（UnitId、FixedVec2、SoldierType、ShieldState、Faction、SeekScope、SeekDirective）SHALL 派生 `Serialize` 和 `Deserialize`。CommandBuffer SHALL 派生 `Serialize` 和 `Deserialize`。

#### Scenario: 完整 Action 枚举序列化
- **WHEN** 创建每个 Action 变体的 GameCommand
- **THEN** 序列化和反序列化后值与原值相等

### Requirement: 确定性随机数

`DeterministicRng` SHALL 提供 `gen_probability_permyriad() -> u32` 方法，返回 0..10000 的确定性概率值（万分比）。原 `gen_probability() -> f32` SHALL 保留但标记为 deprecated，simulation 层 SHALL NOT 使用。

#### Scenario: 万分比概率确定性
- **WHEN** 使用相同 seed 调用 gen_probability_permyriad() 10000 次
- **THEN** 两次运行产生完全相同的 u32 序列

### Requirement: 仿真层依赖限制

simulation crate 的 serde 依赖 SHALL 保持为已有版本，不引入新外部依赖。replay 模块（ReplayFile）SHALL 作为纯数据定义存在，不包含任何 I/O 或引擎概念。

#### Scenario: replay 模块无引擎依赖
- **WHEN** 编译 simulation crate
- **THEN** replay 模块不引入 bevy_render、bevy_window 或任何图形/音频 crate

### Requirement: 战斗系统

战斗系统中所有概率判定（穿透、闪避、格挡、多重射击）SHALL 使用万分比整数。所有比例计算（吸血率、建筑伤害比）SHALL 使用整数乘除。

#### Scenario: 穿透判定使用万分比
- **WHEN** 箭矢穿透概率配置为 3000（30%）
- **AND** gen_probability_permyriad() 返回 2500
- **THEN** 穿透成功（2500 < 3000）

#### Scenario: 骑兵闪避使用万分比
- **WHEN** 骑兵 HP 比例为 80%（8000/10000），最大闪避率为 5000（50%）
- **THEN** 闪避概率为 `(8000 * 5000) / 10000 / 10000 = 4000`（40%）

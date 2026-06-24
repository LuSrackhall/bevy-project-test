## ADDED Requirements

### Requirement: 概率值使用万分比整数

simulation 层所有概率判定 SHALL 使用 `u32` 万分比（0-10000）表示，不得使用 `f32`。`DeterministicRng` SHALL 提供 `gen_probability_permyriad() -> u32` 方法返回 0..10000 的确定性概率值。

#### Scenario: 概率判定确定性
- **WHEN** 使用相同 seed 初始化两个 DeterministicRng 实例
- **AND** 两个实例各调用 10000 次 `gen_probability_permyriad()`
- **THEN** 两个实例产生完全相同的 u32 序列

#### Scenario: 万分比精度覆盖配置需求
- **WHEN** 配置文件中设置 `pierce_chance: 3000`（表示 30.00%）
- **AND** 运行仿真判定穿透
- **THEN** 穿透概率精确为 30.00%，误差不超过 0.01%

### Requirement: 比例计算使用整数乘除

simulation 层所有比例/倍率计算 SHALL 使用整数乘法 + 万分比除法，不得使用 `f32` 乘除。

#### Scenario: 伤害比例计算确定性
- **WHEN** 攻击力为 100，伤害比例配置为 7500（75%）
- **THEN** 实际伤害为 `(100 * 7500) / 10000 = 75`，结果为整数

#### Scenario: 减速倍率幂运算确定性
- **WHEN** 减速基础值为 8000（80%），叠加 3 层
- **THEN** 最终倍率为 `10000 * 8000 / 10000 * 8000 / 10000 * 8000 / 10000 = 5120`（51.2%），使用循环整数乘法

### Requirement: 仿真层无 f32 仿真运算

simulation crate 中除 presentation 桥接方法（`from_float`/`to_float`）外，SHALL 不存在任何 `f32`/`f64` 参与仿真逻辑分支判定或数值计算的代码。

#### Scenario: 编译期检查
- **WHEN** 在 simulation crate 中搜索 `as f32`、`as f64`、`: f32`、`: f64` 使用
- **THEN** 仅在 `types.rs` 的 `from_float`/`to_float` 桥接方法和 `gen_probability()`（标记为 deprecated）中存在

#### Scenario: 确定性判定链无浮点
- **WHEN** 从 gen_probability_permyriad() 获取概率值到最终战斗结果
- **THEN** 整个判定链中不存在任何 f32/f64 运算

### Requirement: BTreeMap 替代 HashMap 用于迭代顺序敏感场景

simulation 层中影响仿真结果的集合迭代 SHALL 使用 `BTreeMap`（按 key 排序），不得使用 `HashMap`（迭代顺序不确定）。

#### Scenario: 最近敌人扫描确定性
- **WHEN** 存在两个距离完全相同的敌人（UnitId 不同）
- **THEN** 每次扫描选择的目标一致（按 UnitId 排序优先）

### Requirement: RNG 版本锁定

`rand` crate 版本 SHALL 通过 Cargo.lock 锁定，确保 SmallRng 算法在不同构建间一致。

#### Scenario: RNG 版本一致性
- **WHEN** 使用相同 Cargo.lock 构建两个二进制
- **THEN** 两个二进制的 SmallRng 产生完全相同的随机序列

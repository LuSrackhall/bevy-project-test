## ADDED Requirements

### Requirement: FactionId 强类型

`FactionId` 是 `simulation::types` 中定义的独立强类型，用于标识单位/城市所属的阵营。取代现有的 `Faction` 枚举。

```rust
pub struct FactionId(pub u8);
```

- `FactionComponent` 的类型从 `FactionComponent(pub Faction)` 改为 `FactionComponent(pub FactionId)`
- 所有 `Faction::Player` 引用替换为 `FactionId(0)`，`Faction::Enemy` 替换为 `FactionId(1)`，`Faction::Neutral` 替换为 `FactionId(2)`
- `FactionId` 无 `Default` impl，必须显式构造
- `FactionId` 实现 `Clone`、`Copy`、`PartialEq`、`Eq`、`Debug`、`Hash`、`Serialize`、`Deserialize`

#### Scenario: FactionId 构造与比较

- **WHEN** 创建 `FactionId(0)` 和 `FactionId(0)`
- **THEN** 两者相等

#### Scenario: FactionId 赋值给 FactionComponent

- **WHEN** `FactionComponent(FactionId(0))` 赋值给实体组件
- **THEN** 实体创建成功，组件可查询

### Requirement: FactionId 替换现有所有 Faction 枚举引用

代码库中所有 `Faction::Player`、`Faction::Enemy`、`Faction::Neutral` 引用必须替换为 `FactionId(0)`、`FactionId(1)`、`FactionId(2)`。

#### Scenario: Faction 枚举删除

- **WHEN** 删除 `Faction` 枚举定义
- **THEN** 编译器报告所有未迁移的引用位置

### Requirement: TeamId 强类型

`TeamId` 是 `simulation::types` 中定义的独立强类型，标识阵营所属的队伍（胜负关系分组）。

```rust
pub struct TeamId(pub u8);
```

- `TeamId` 无 `Default` impl
- `TeamId` 实现 `Clone`、`Copy`、`PartialEq`、`Eq`、`Debug`、`Hash`

#### Scenario: TeamId 构造

- **WHEN** 构造 `TeamId(0)`
- **THEN** `TeamId(0)` 可用作胜利条件判定的分组键

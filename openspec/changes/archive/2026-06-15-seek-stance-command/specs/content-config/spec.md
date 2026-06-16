## MODIFIED Requirements

### Requirement: units.ron 兵种配置

`content/units.ron` SHALL 定义每个兵种的关键属性：`health`（生命值）、`attack`（基础攻击力）、`speed`（移动速度）、`attack_range`（攻击距离）、`attack_interval_ticks`（攻击间隔 Tick 数）、`spawn_speed_mult`（产兵速度倍率）。配置 SHALL 以兵种 key 为顶级键（`militia`、`infantry`、`archer`、`cavalry`）。`aggression_range` 字段 SHALL 被移除，主动索敌行为改为由 `Action::SetSeekStance` 命令控制。

#### Scenario: 民兵配置

- **WHEN** 读取 `content/units.ron` 中 `militia` 条目
- **THEN** `health = 100`, `attack = 16`, `speed = 80`, `attack_range = 30`, `attack_interval_ticks = 10`, `spawn_speed_mult = 1.0`
- **AND** 不包含 `aggression_range` 字段

#### Scenario: 弓兵配置

- **WHEN** 读取 `content/units.ron` 中 `archer` 条目
- **THEN** `attack_range = 600`，不包含 `aggression_range` 字段

#### Scenario: 骑兵配置

- **WHEN** 读取 `content/units.ron` 中 `cavalry` 条目
- **THEN** `speed = 200`, `health = 140`，不包含 `aggression_range` 字段

## REMOVED Requirements

### Requirement: aggression_range 被动索敌字段

**Reason**: 主动索敌行为从被动配置属性改为玩家主动下发的状态命令（`SeekStance` 组件 + `Action::SetSeekStance` 命令），`aggression_range` 字段不再需要。

**Migration**: 从 `content/units.ron` 中删除每个兵种的 `aggression_range` 行。`SoldierUnitConfig` 结构体中移除该字段，添加 `#[serde(default)]` 以兼容旧配置文件。`combat_engagement_system` 改用 `SeekStance.seek_range` 替代 `aggression_range`。

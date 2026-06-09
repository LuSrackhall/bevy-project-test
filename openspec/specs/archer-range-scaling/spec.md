# archer-range-scaling

## Purpose

弓箭手攻击距离随等级从 380 增长到 600 内部单位，超射距离同步增长。箭速通过配置定义，飞行 Tick 在游戏启动时预计算。

## Requirements

### Requirement: 攻击距离随等级成长

弓箭手的攻击距离 SHALL 从 Lv.1 的 380 内部单位线性增长到 Lv.4 的 600 内部单位。公式：`attack_range = 380 + (level - 1) × 220 / 3`。

#### Scenario: 1 级弓兵射程

- **WHEN** 弓兵等级为 1
- **THEN** 攻击距离 = 380 内部单位

#### Scenario: 4 级弓兵最大射程

- **WHEN** 弓兵等级为 4
- **THEN** 攻击距离 = 600 内部单位

#### Scenario: 2 级弓兵中间射程

- **WHEN** 弓兵等级为 2
- **THEN** 攻击距离 = 453 内部单位（380 + 73）

### Requirement: 超射距离随等级成长

箭矢飞行距离 SHALL = 攻击距离 + 超射距离。超射距离公式：`20 + (level - 1) × 20`。

#### Scenario: 1 级箭矢飞行距离

- **WHEN** 1 级弓兵发射箭矢
- **THEN** 总飞行距离 = 380 + 20 = 400 内部单位

#### Scenario: 4 级箭矢飞行距离

- **WHEN** 4 级弓兵发射箭矢
- **THEN** 总飞行距离 = 600 + 80 = 680 内部单位

### Requirement: 箭矢速度配置

箭矢飞行速度 SHALL 通过 `content/units.ron` 中 archer 兵种的 `arrow_speed` 字段配置（内部单位/Tick）。

#### Scenario: 默认箭速

- **WHEN** 加载 `content/units.ron`
- **THEN** archer 的 `arrow_speed` = 20（默认值）

### Requirement: 飞行 Tick 预计算

最大飞行 Tick 数 SHALL 在 `init_simulation_world` 时根据最大攻击距离和最大超射一次性计算，运行时直接使用预计算值。

#### Scenario: Tick 预计算

- **WHEN** `init_simulation_world(seed)` 执行
- **THEN** 计算出 `max_flight_ticks = (600 + 80) / 20 = 34`，存储为全局常量供 archer_attack_system 使用

### Requirement: 攻击范围配置字段定义

`SoldierUnitConfig` SHALL 新增以下字段用于运行时计算攻击范围：`arrow_speed: u32`、`attack_range_base: u32`、`attack_range_max: u32`、`attack_range_max_level: u32`、`overshoot_base: u32`、`overshoot_per_level: u32`、`pierce_base_chance: f32`、`pierce_per_level: f32`、`pierce_unlock_level: u32`。

#### Scenario: 从 RON 加载

- **WHEN** 解析 `content/units.ron` 中 archer 条目
- **THEN** 上述所有字段正确反序列化为 Rust 类型

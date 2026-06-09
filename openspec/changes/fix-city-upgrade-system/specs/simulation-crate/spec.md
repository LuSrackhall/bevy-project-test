## MODIFIED Requirements

### Requirement: 城池组件与系统

simulation crate SHALL 定义城池相关组件：`CityComponent`（等级/人口/经验/产兵类型/冷却）、`CityRadius`、`AuraHeal`。SHALL 提供 `city_spawn_system` 按冷却间隔增加人口并生成士兵实体。SHALL 提供 `city_interaction_system` 处理士兵进城逻辑（攻城/治疗/升级）。城池升级时 SHALL 同步更新 `level`、`health_max`、`health_current`、`max_population`、`CityRadius`。

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

#### Scenario: 城池升级

- **WHEN** 友方士兵以城池为目标进入城内，城池满血且 `level < max_level`
- **THEN** `level_exp += max_hp × level_up_gain_ratio`。若 `level_exp >= max_hp × level_up_cost_multiplier × level`：`level += 1`，`health_max = level × level_hp_multiplier`，`health_current = health_max`，`max_population = level × base_population_per_level`，`CityRadius = visual_radius_base + level × visual_radius_per_level`

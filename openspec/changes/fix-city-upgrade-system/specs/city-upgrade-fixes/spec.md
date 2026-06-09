## ADDED Requirements

### Requirement: 城池升级时同步更新人口上限

城池升级时（`city_interaction_system` 中 `level += 1` 分支）SHALL 同步更新 `max_population` 为 `level × base_population_per_level`。升级后的城池 SHALL 能容纳与等级匹配的人口。

#### Scenario: Lv.1 升 Lv.2 人口上限增长

- **WHEN** 城池从 Lv.1 升级到 Lv.2
- **THEN** `max_population` 从初始值更新为 `2 × base_population_per_level`（默认 24）

#### Scenario: Lv.5 城池人口上限

- **WHEN** 城池等级为 5
- **THEN** `max_population = 5 × 12 = 60`

### Requirement: 城池升级时同步更新视觉半径

城池升级时 SHALL 同步更新 `CityRadius` 组件，值为 `visual_radius_base + level × visual_radius_per_level`。升级后的城池 SHALL 具有与等级匹配的碰撞和视觉半径。

#### Scenario: 升级后半径增长

- **WHEN** 城池从 Lv.1（CityRadius = 18）升级到 Lv.3
- **THEN** `CityRadius` 更新为 `15 + 3 × 3 = 24`

#### Scenario: 降级后半径缩小

- **WHEN** 城池被占领导致降级（Lv.3 → Lv.2）
- **THEN** `CityRadius` 更新为 `15 + 2 × 3 = 21`

### Requirement: 升级成本线性增长

城池升级所需士兵数 SHALL 随等级线性增长。公式：`required_exp = max_hp × level_up_cost_multiplier × level`，`gain = max_hp × level_up_gain_ratio`。默认 `cost_multiplier = 1.0`、`gain_ratio = 0.30`，`soldiers_needed ≈ 3.33 × level`。公式 SHALL NOT 包含硬编码等级上限。

#### Scenario: Lv.1 升 Lv.2 成本

- **WHEN** `cost_multiplier = 1.0`，`gain_ratio = 0.30`，城池 Lv.1（max_hp = 100）
- **THEN** 每个士兵贡献 `100 × 0.30 = 30` exp，需要 `100 × 1.0 × 1 = 100` exp，约 4 个士兵

#### Scenario: Lv.5 升 Lv.6 成本

- **WHEN** 城池 Lv.5（max_hp = 500）
- **THEN** 每个士兵贡献 `500 × 0.30 = 150` exp，需要 `500 × 1.0 × 5 = 2500` exp，约 17 个士兵

#### Scenario: 高等级线性增长

- **WHEN** 城池 Lv.20（max_hp = 2000）
- **THEN** 每个士兵贡献 `2000 × 0.30 = 600` exp，需要 `2000 × 1.0 × 20 = 40000` exp，约 67 个士兵

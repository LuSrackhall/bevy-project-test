## ADDED Requirements

### Requirement: RON 格式外置配置

content 目录 SHALL 包含 4 个 `.ron` 配置文件：`units.ron`（兵种属性）、`cities.ron`（城池参数）、`combat.ron`（战斗公式参数）、`map.ron`（地图生成参数）。所有文件 SHALL 以注释块开头，说明该文件的配置格式、字段含义、取值范围和修改方式。

#### Scenario: 配置文件存在且格式正确

- **WHEN** 游戏启动，simulation 初始化
- **THEN** 4 个 `.ron` 文件均被成功解析为对应的 Rust 结构体，数值与文件中定义一致

#### Scenario: 注释说明可读

- **WHEN** 开发者用文本编辑器打开 `content/units.ron`
- **THEN** 文件前 20 行内包含注释说明：兵种 key 命名规则、每个字段的单位和取值范围、如何新增兵种
- **AND** 每个兵种条目内有行内注释解释关键字段

### Requirement: units.ron 兵种配置

`content/units.ron` SHALL 定义每个兵种的关键属性：`health`（生命值）、`attack`（基础攻击力）、`speed`（移动速度）、`attack_range`（攻击距离）、`aggression_range`（主动索敌距离）、`attack_interval_ticks`（攻击间隔 Tick 数）、`spawn_speed_mult`（产兵速度倍率）。配置 SHALL 以兵种 key 为顶级键（`militia`、`infantry`、`archer`、`cavalry`）。

#### Scenario: 民兵配置

- **WHEN** 读取 `content/units.ron` 中 `militia` 条目
- **THEN** `health = 100`, `attack = 16`, `speed = 80`, `attack_range = 30`, `aggression_range = 150`, `attack_interval_ticks = 10`, `spawn_speed_mult = 1.0`

#### Scenario: 弓兵配置

- **WHEN** 读取 `content/units.ron` 中 `archer` 条目
- **THEN** `attack_range = 600`, `aggression_range = 600`（等于攻击范围）

#### Scenario: 骑兵配置

- **WHEN** 读取 `content/units.ron` 中 `cavalry` 条目
- **THEN** `speed = 200`, `health = 140`, `aggression_range = 200`（快速、高生命、广索敌）

### Requirement: cities.ron 城池配置

`content/cities.ron` SHALL 定义城池相关的所有参数：`level_hp_multiplier`、`base_population_per_level`、`visual_radius_base`、`visual_radius_per_level`、`heal_ratio`、`level_up_cost_multiplier`、`level_up_gain_ratio`、`capture_hp_ratio`、`aura` 子对象（`base_radius`、`spawn_dir_radius`、`base_heal`、`per_level_heal`）。

#### Scenario: 城池 HP 计算

- **WHEN** 城池 level = 3，`level_hp_multiplier = 100`
- **THEN** `max_health = 3 * 100 = 300`

#### Scenario: 光环治疗计算

- **WHEN** 城池 level = 5，光环内有一个友方士兵
- **THEN** 每 Tick 治疗量 = `base_heal + (level - 1) * per_level_heal`

#### Scenario: 占领后 HP 恢复

- **WHEN** 城池被占领
- **THEN** 新 HP = `max_health * capture_hp_ratio`

### Requirement: combat.ron 战斗配置

`content/combat.ron` SHALL 定义战斗相关参数：`city_damage_per_soldier_ratio`、`archer_melee_range`、`archer_melee_damage_mult`、`shield` 子对象、`cavalry` 子对象、`archer_multi_shot` 子对象、`slow_debuff` 子对象、`level_up` 子对象、`fearless` 子对象。

#### Scenario: 弓兵近战距离

- **WHEN** 弓箭手与目标距离 <= `archer_melee_range`（配置值 50 像素）
- **THEN** 弓箭手造成的伤害乘以 `archer_melee_damage_mult`（配置值 0.85）

#### Scenario: 步兵举盾减伤

- **WHEN** 步兵 `ShieldState == ShieldUp`，且箭矢来自前方（点积 > 0）
- **THEN** 有 `shield.intercept_chance` 概率触发减伤，伤害乘以 `1 - shield.damage_reduction`

#### Scenario: 减速叠层

- **WHEN** 单位被新箭矢击中，已有 2 层 SlowDebuff
- **THEN** 层数变为 3（上限 `slow_debuff.max_stacks = 9`），减速系数 = `slow_amount * stack_mult^(stacks-1)` 且不低于 `max_reduction`
- **AND** 持续时间重置为 `slow_debuff.duration_ticks`

#### Scenario: 升级后解锁吸血

- **WHEN** 单位 `level >= level_up.lifesteal_unlock_level`（配置值 4）
- **THEN** `lifesteal_rate`（配置值 0.10）对每次成功攻击生效，回复 `damage * lifesteal_rate` 生命值

#### Scenario: 无畏触发

- **WHEN** 骑兵闪避成功
- **THEN** 骑兵获得 `FearlessBuff` 持续 `fearless.duration_ticks`（配置值 40 = 2秒@20Hz），期间 `attack += fearless.attack_bonus`，`lifesteal_rate += fearless.lifesteal_bonus`

### Requirement: map.ron 地图配置

`content/map.ron` SHALL 定义地图生成参数：`width`、`height`、`min_cities`、`max_cities`、`city_min_distance`、`margin`、`neutral_city_ratio`（[min, max] 范围）、`city_level_range`（[min, max] 范围）。

#### Scenario: 城池数量在范围内

- **WHEN** 使用 `min_cities = 6`，`max_cities = 20` 生成地图
- **THEN** 生成的城池数在 [6, 20] 范围内

#### Scenario: 城池间距满足最小距离

- **WHEN** `city_min_distance = 250`
- **THEN** 任意两座城池之间的距离 >= 250 像素（或达到最大尝试次数后放宽）

#### Scenario: 边距限制

- **WHEN** `margin = 150`，地图 `width = 2000`
- **THEN** 所有城池的 x 坐标在 [150, 1850] 范围内

### Requirement: 配置文件版本化

content 目录 SHALL 被 git 追踪。配置修改后 SHALL 通过 git diff 可审查。SHALL NOT 依赖任何非 git 的外部版本控制系统。

#### Scenario: 配置变更可追踪

- **WHEN** 开发者修改 `content/combat.ron` 中的 `shield.damage_reduction` 从 0.80 改为 0.70
- **THEN** `git diff` 显示该变更，可被 Code Review 审查

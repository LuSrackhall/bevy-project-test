## Purpose
城池交互系统：选中、视觉更新、占领翻转、士兵消耗行为。
## Requirements
### Requirement: Click to select player city
玩家在 Playing 状态下左键点击（桌面）/单指点击（移动端）己方城池时，SHALL 发出 CitySelectedEvent。

#### Scenario: Player clicks own city
- **WHEN** 玩家左键（桌面）或单指（移动端）点击己方城池（点击位置世界坐标在城池 visual_radius 内）
- **THEN** 发出 CitySelectedEvent{entity, faction: Player}，底部面板显示该城池数据

#### Scenario: Player clicks enemy city
- **WHEN** 玩家点击敌方城池
- **THEN** 不发出 CitySelectedEvent（只有己方城池可选中查看）

#### Scenario: Player clicks empty ground
- **WHEN** 玩家点击地图空白区域（无城池命中）
- **THEN** 不发出 CitySelectedEvent，底部面板隐藏

### Requirement: City visual updates on faction change
城池占领后，其视觉颜色 SHALL 随 faction 变更实时刷新。Level 变化时圆环半径也需更新。

#### Scenario: Enemy captures player city
- **WHEN** 敌方占领己方城池（faction 从 Player 变为 Enemy）
- **THEN** 城池圆环 Fill 颜色从蓝色变为红色，圆环半径按新 level 重新计算

### Requirement: Neutral city flips to attacker on capture
中立城池被攻击且 HP ≤ 0 时，SHALL 翻转为攻击方阵营。

#### Scenario: Player attacks neutral city to death
- **WHEN** 中立城池 HP 被玩家士兵攻击至 ≤ 0
- **THEN** 城池 faction 变为 Player，HP 恢复为 20%，level 保持原值

### Requirement: City does not consume soldier when at max health and max level
当己方城池满血（`health_current >= health_max`）且满级（`level >= max_level`）时，到达城池范围内的目标士兵 SHALL NOT 被 despawn，士兵 SHALL 停留在当前位置。

#### Scenario: Soldier arrives at maxed city
- **WHEN** 己方士兵移动目标为己方城池，到达城池范围（距离 ≤ city_radius + 5），且城池 health_current >= health_max 且 level >= max_level
- **THEN** 士兵不被销毁，保持当前位置，不扣减出生城人口

#### Scenario: Soldier arrives at damaged city
- **WHEN** 己方士兵移动目标为己方城池，到达城池范围，且城池 health_current < health_max
- **THEN** 士兵按原有规则治愈城池后被 despawn，扣减出生城人口

#### Scenario: Soldier arrives at under-leveled city
- **WHEN** 己方士兵移动目标为己方城池，到达城池范围，且城池满血但 level < max_level
- **THEN** 士兵按原有规则贡献升级经验后被 despawn，扣减出生城人口


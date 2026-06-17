## ADDED Requirements

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

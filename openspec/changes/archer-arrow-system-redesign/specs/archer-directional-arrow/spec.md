## ADDED Requirements

### Requirement: 箭矢沿固定方向飞行

箭矢 SHALL 在发射时计算固定方向向量（指向目标位置），每 Tick 沿该方向前进 `speed` 距离。箭矢 SHALL NOT 跟踪目标位置变化。

#### Scenario: 箭矢直线飞行

- **WHEN** 弓兵在 (0,0) 向 (600,0) 方向的目标发射箭矢
- **THEN** 每 Tick 箭矢位置沿 X 轴正方向前进 speed 距离，不受目标移动影响

#### Scenario: 目标移动不影响箭矢方向

- **WHEN** 箭矢已发射，目标移动到新位置
- **THEN** 箭矢继续沿原始方向飞行，不改变航向

### Requirement: 箭矢飞行阶段碰撞检测

箭矢在 `flight_remaining > 0` 阶段 SHALL 每 Tick 检测与所有敌方单位的碰撞。当箭矢位置与敌方单位逻辑位置的距离 < 碰撞半径时，SHALL 造成伤害。

#### Scenario: 箭矢命中敌方单位

- **WHEN** 箭矢飞行至敌方单位碰撞半径内
- **THEN** 对目标造成 Arrow.damage 点伤害

#### Scenario: 箭矢不命中友方

- **WHEN** 箭矢穿过友方单位位置
- **THEN** 不造成任何伤害

### Requirement: 穿透机制

Lv.2 及以上弓兵发射的箭矢 SHALL 具有穿透几率。发射时掷一次骰子：若 `< pierce_chance`，碰撞后继续飞行；否则进入衰减阶段。

#### Scenario: 穿透成功继续飞行

- **WHEN** 3 级弓兵箭矢命中第一个敌人且穿透判定通过
- **THEN** 箭矢继续飞行，可碰撞后续敌人。命中过的单位加入 hit_units，不会再被同一箭矢命中

#### Scenario: 穿透失败进入衰减

- **WHEN** 箭矢命中敌人且穿透判定未通过
- **THEN** stuck_to 设为该目标，decay_remaining 设为 20 Tick，箭矢不再造成伤害

#### Scenario: 1 级弓兵无穿透

- **WHEN** 1 级弓兵箭矢命中任何敌人
- **THEN** 穿透率为 0%，必定进入衰减

### Requirement: 衰减阶段

`decay_remaining > 0` 阶段 SHALL 为纯视觉效果：箭矢跟随 `stuck_to` 目标移动（若有）或静止在最终位置，直至 ticks 耗尽后销毁。该阶段 SHALL NOT 造成任何碰撞伤害。

#### Scenario: 衰减阶段跟随目标

- **WHEN** 箭矢命中目标进入衰减且 stuck_to 存在
- **THEN** 每 Tick 箭矢位置更新为 stuck_to 目标的当前位置

#### Scenario: 自然衰减

- **WHEN** 箭矢 flight_remaining 耗尽且未命中任何敌人
- **THEN** decay_remaining 设为 20 Tick，箭矢在当前位置进入衰减，不跟随任何目标

#### Scenario: 衰减期满销毁

- **WHEN** decay_remaining 递减至 0
- **THEN** 箭矢实体被销毁

### Requirement: 箭矢生命周期预计算

箭矢飞行 Tick 数 SHALL 在游戏启动时预计算，基于配置中的最大攻击距离、最大超射距离和箭速。飞行中仅递减计数器，不进行除法运算。

#### Scenario: 预计算值存储于配置

- **WHEN** `init_simulation_world` 执行
- **THEN** `max_flight_ticks = (attack_range_max + overshoot_max) / arrow_speed` 作为固定常量存储，运行时直接使用

### Requirement: 碰撞防重复

同一箭矢 SHALL NOT 对同一单位造成多次伤害。`hit_units` 字段 SHALL 记录已命中的 UnitId，碰撞检测时跳过。

#### Scenario: 箭矢不重复命中

- **WHEN** 穿透箭矢连续两 Tick 经过同一敌方单位
- **THEN** 仅第一次检测到碰撞时造成伤害，后续 ticks 跳过该单位

### Requirement: 战斗状态互斥

弓兵 SHALL 在攻击冷却恢复且射程内有敌人时进入 Fighting 状态并发射箭矢，SHALL NOT 在 Fighting 状态下移动。射程内无敌人时 SHALL 恢复 Moving 状态。

#### Scenario: 有敌人时射击

- **WHEN** 弓兵攻击冷却为 0 且射程内存在敌方单位
- **THEN** 弓兵发射箭矢，重置冷却，保持 Fighting 状态

#### Scenario: 无敌人时恢复移动

- **WHEN** 弓兵 Fighting 状态下，射程内没有任何敌方单位
- **THEN** 弓兵状态变为 Moving，恢复移动

### Requirement: 弓兵不参与自动索敌

弓兵 SHALL NOT 被 `combat_engagement_system` 处理。弓兵仅在其当前攻击范围内检测敌人并射击，不会自动向敌人移动。

#### Scenario: 弓兵不自动追击

- **WHEN** 敌方单位在弓兵 aggression_range 内但不在 attack_range 内
- **THEN** combat_engagement_system 跳过弓兵，弓兵不设置 Movement.target

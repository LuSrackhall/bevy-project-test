## ADDED Requirements

### Requirement: 箭矢与城池建筑碰撞检测

箭矢在飞行阶段（`flight_remaining > 0`）SHALL 检测与敌方城池建筑的碰撞。当箭矢位置与城池中心距离 < `CityRadius` 时，SHALL 判定为命中。城池碰撞检测 SHALL 在兵种碰撞检测之后执行，仅当箭矢未被兵种碰撞停止（即穿透成功或无兵种命中）时进行。

#### Scenario: 箭矢命中敌方城池

- **WHEN** 箭矢飞行至敌方城池的 `CityRadius` 范围内
- **THEN** 城池 `CityComponent.arrow_damage_acc += arrow.damage`，若累加值 >= 200 则每满 200 扣 1 点 `health_current`（余数保留），箭矢进入衰减阶段（decay）

#### Scenario: 箭矢穿过己方城池

- **WHEN** 箭矢进入己方阵营城池的 `CityRadius` 范围
- **THEN** 不发生碰撞检测，箭矢继续飞行

#### Scenario: 箭矢穿透兵种后命中后方城池

- **WHEN** 箭矢在同一 Tick 内穿透命中第一个敌方单位后，继续飞行并进入后方敌方城池的 `CityRadius`
- **THEN** 城池碰撞检测生效，执行建筑伤害逻辑

#### Scenario: 箭矢被兵种停止后不检查城池

- **WHEN** 箭矢命中敌方兵种且穿透判定失败（进入 decay）
- **THEN** 同一 Tick 内不再检查城池碰撞

### Requirement: 箭矢对建筑的伤害计算

箭矢对建筑的每次碰撞伤害 SHALL 以 `1 / arrow_building_damage_denominator` 比例计算（默认分母 200）。`CityComponent.arrow_damage_acc` 字段（u32）SHALL 作为伤害累加器，每次命中累加 `arrow.damage` 值。累加值每达到分母整倍数时 SHALL 对 `health_current` 造成 1 点伤害并扣减分母值，余数 SHALL 保留以供后续命中继续累加。该机制 SHALL 保证零舍入误差，全整数确定性运算。

#### Scenario: 低伤害箭矢累积多次后扣血

- **WHEN** 攻击力 16 的箭矢命中敌方城池 13 次
- **THEN** `arrow_damage_acc = 208`，触发 1 次扣血，`health_current -= 1`，`arrow_damage_acc = 8`

#### Scenario: 高伤害箭矢单次命中即扣血

- **WHEN** 攻击力 200 的箭矢命中敌方城池 1 次
- **THEN** `arrow_damage_acc = 200`，触发 1 次扣血，`health_current -= 1`，`arrow_damage_acc = 0`

#### Scenario: 箭矢伤害极低不扣血

- **WHEN** 攻击力 10 的箭矢命中敌方城池 1 次
- **THEN** `arrow_damage_acc = 10`，不触发扣血（10 < 200）

### Requirement: 建筑碰撞后箭矢强制衰减

箭矢命中城池建筑后 SHALL 强制进入衰减阶段（`decay_remaining = ARROW_DECAY_TICKS`）。SHALL NOT 进行穿透判定（`pierce_chance` 仅适用于兵种）。衰减阶段与现有规则一致：不再造成伤害，跟随目标或静止直至 ticks 耗尽后销毁。

#### Scenario: 命中建筑后无穿透判定

- **WHEN** 5 级弓兵（高穿透率）的箭矢命中敌方城池
- **THEN** 箭矢直接进入衰减，不执行穿透判定

#### Scenario: 命中建筑后的衰减行为

- **WHEN** 箭矢命中敌方城池后进入衰减
- **THEN** `decay_remaining = 20`，每 Tick 递减，耗尽后箭矢实体被销毁。衰减期间不造成任何碰撞伤害

### Requirement: 城池伤害累加器初始化

新生成的城池实体 SHALL 将 `CityComponent.arrow_damage_acc` 初始化为 0。城池易手后 SHALL 将 `arrow_damage_acc` 重置为 0。

#### Scenario: 新城池累加器为零

- **WHEN** 地图生成系统创建新城池
- **THEN** `CityComponent.arrow_damage_acc == 0`

#### Scenario: 城池易手后累加器重置

- **WHEN** 城池被敌方占领（faction 变更）
- **THEN** `arrow_damage_acc` 重置为 0

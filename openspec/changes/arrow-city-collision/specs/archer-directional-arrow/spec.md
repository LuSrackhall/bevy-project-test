## MODIFIED Requirements

### Requirement: 箭矢飞行阶段碰撞检测

箭矢在 `flight_remaining > 0` 阶段 SHALL 每 Tick 检测与所有敌方单位的碰撞。当箭矢位置与敌方单位逻辑位置的距离 < 碰撞半径时，SHALL 造成伤害。兵种碰撞检测完成后，若箭矢未被兵种停止（穿透成功或无兵种命中），SHALL 继续进行城池建筑碰撞检测。当箭矢位置与敌方城池中心距离 < `CityRadius` 时 SHALL 判定为建筑命中。己方城池 SHALL NOT 触发碰撞。

#### Scenario: 箭矢命中敌方单位

- **WHEN** 箭矢飞行至敌方单位碰撞半径内
- **THEN** 对目标造成 Arrow.damage 点伤害

#### Scenario: 箭矢不命中友方

- **WHEN** 箭矢穿过友方单位位置
- **THEN** 不造成任何伤害

#### Scenario: 箭矢命中敌方城池建筑

- **WHEN** 箭矢飞行至敌方城池 `CityRadius` 范围内且未因兵种停止
- **THEN** 对城池建筑以 1/200 累积比例造成伤害，箭矢进入衰减阶段（不穿透）

#### Scenario: 箭矢穿过己方城池

- **WHEN** 箭矢进入己方城池 `CityRadius` 范围
- **THEN** 不发生碰撞，箭矢继续飞行

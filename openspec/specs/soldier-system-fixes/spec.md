## ADDED Requirements

### Requirement: Soldier-city interaction decoupled from capture
士兵进城操作（扣血/回血/升级）SHALL 与城池占领判定分离。士兵进城后 despawn，城池 HP 扣减（敌方）或回复/升级（己方）。占领判定 SHALL 由独立的 city_capture_system 在每帧统一检测 HP ≤ 0 时执行。

#### Scenario: Soldier enters enemy city but doesn't kill it
- **WHEN** 己方士兵进入敌方城池，造成的伤害使城池 HP 降至 30/100（未 ≤ 0）
- **THEN** 士兵 despawn，城池 HP = 30，城池 faction 不变。后续 frame 中 city_capture_system 检测 HP > 0，不触发占领

#### Scenario: Soldier enters enemy city and kills it
- **WHEN** 己方士兵进入敌方城池，造成的伤害使城池 HP 降至 -5/100（≤ 0）
- **THEN** 士兵 despawn，城池 HP = -5。下一帧 city_capture_system 检测 HP ≤ 0，触发占领（翻转 faction、降级、回血 20%）

### Requirement: Exiled soldiers survive city capture
被攻陷方由该城产出的士兵 SHALL 变为流亡士兵（is_exiled = true），保留在地图上，归属不变，可继续战斗，不死亡。

#### Scenario: Player captures enemy city
- **WHEN** city_capture_system 翻转敌方城池为 Player
- **THEN** CityCapturedEvent 触发 → observer 将 city_origin == 该城池 Entity 的敌方士兵标记 is_exiled = true。这些士兵保留在地图上，仍属 Enemy faction

#### Scenario: Exiled soldiers not counted in population
- **WHEN** 流亡士兵的产出城池已丢失
- **THEN** 流亡士兵不占用任何城池的人口名额（city_spawn_system 的人口计数不受影响）

### Requirement: City aura spawn-direction bonus is dynamic
城池光环的出兵走廊方向 SHALL 从 City 的 spawn_direction 字段读取，而非硬编码 Vec2::X。

#### Scenario: City spawns soldiers toward nearest enemy
- **WHEN** 城池 spawn_direction 为 (0, -1)（朝向下方敌方城池）
- **THEN** 光环的出兵走廊方向为 (0, -1)，处于该方向 300px 范围内的己方士兵获得额外治疗

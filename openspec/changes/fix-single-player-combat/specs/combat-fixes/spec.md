## ADDED Requirements

### Requirement: Archer multi-shot targets different enemies
弓兵触发多重射击时，每支箭 SHALL 射向攻击范围内不同的敌方单位，而非同一目标。

#### Scenario: Multi-shot with 3+ enemies in range
- **WHEN** 弓兵攻击间隔触发且多重射击 roll 成功（目标数 N=3），攻击范围内存在 4 个敌方单位
- **THEN** 生成 3 支 Arrow，分别射向距离最近的 3 个不同敌方 Entity

#### Scenario: Multi-shot with fewer enemies than target count
- **WHEN** 弓兵多重射击目标数 N=4，但攻击范围内只有 2 个敌方单位
- **THEN** 生成 2 支 Arrow，分别射向这 2 个敌方 Entity

### Requirement: Arrow has visual rendering
箭矢 Entity SHALL 附带可见的 Lyon Shape（小圆形），颜色按阵营区分（己方蓝色、敌方红色），到达目标后随 Entity 一起销毁。

#### Scenario: Archer fires arrow
- **WHEN** 弓兵生成 Arrow Entity
- **THEN** Arrow 附带半径为 3px 的圆形 Fill，颜色与弓兵阵营对应（Player=蓝，Enemy=红）

### Requirement: Slow debuff stacks correctly
弓兵箭矢命中时，减速 debuff SHALL 叠加而非覆盖。若目标已有 SlowDebuff 则 stacks+1（上限保证移速不低于原始速度 35%），刷新 timer 为 1s。无 SlowDebuff 时插入 stacks=1。

#### Scenario: First arrow hit
- **WHEN** 目标无 SlowDebuff，被弓兵箭矢命中
- **THEN** 插入 SlowDebuff{stacks: 1, timer: 1s}，目标移速 ×0.85

#### Scenario: Second arrow hit within debuff duration
- **WHEN** 目标已有 SlowDebuff{stacks: 1}，再次被弓兵箭矢命中
- **THEN** stacks 变为 2，timer 刷新为 1s，目标移速 = 基础移速 × 0.85 × 0.9

#### Scenario: Slow cap prevents excessive reduction
- **WHEN** SlowDebuff stacks 已使移速降至原始速度 35%
- **THEN** 再次命中不再增加 stacks（移速保持 ≥ 原始速度 × 35%）

### Requirement: Slow debuff decays over time
SlowDebuff 的 timer 到期后 SHALL 自动移除 Component，恢复士兵原始移速。

#### Scenario: Slow debuff expires
- **WHEN** SlowDebuff timer 达到 1s 且无新的箭矢命中刷新
- **THEN** SlowDebuff Component 从士兵 Entity 上移除

### Requirement: Melee attack respects attack interval
近战士兵 SHALL 按 attack_timer 间隔执行攻击，而非每帧攻击。attacker 自身的 attack_timer 在 melee_attack_system 中 tick。

#### Scenario: Melee soldier attacks
- **WHEN** 近战士兵（民兵/步兵/骑兵）attack_timer.just_finished() 且目标在攻击范围内
- **THEN** 执行一次攻击（伤害计算 + 闪避判定 + 吸血），其他帧不攻击

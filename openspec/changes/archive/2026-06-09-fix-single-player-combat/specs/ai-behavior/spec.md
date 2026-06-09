## ADDED Requirements

### Requirement: AI defense evaluation
AI 每 2 秒评估己方城池血量。若城池 HP < 50%，SHALL 切换为随机克制兵种，并将以该城为 origin 且在光环范围内的 50% 士兵派往受威胁城池驻守。

#### Scenario: City under 50% health triggers defense
- **WHEN** 敌方城池 Lv.3 HP 降至 80/300（< 50%）
- **THEN** AI 切换该城池 spawn_type 为随机克制兵种，将光环范围内 50% 的该城产出士兵 target 设为该城 Entity（回防）

### Requirement: AI expansion evaluation
AI SHALL 评估最近中立城池。若中立城池 500px 范围内 AI 兵力总攻击力 > 中立城 MaxHealth × 1.5，则派范围内最近的一组士兵进攻该中立城。

#### Scenario: Sufficient force near neutral city
- **WHEN** 中立城池 Lv.2（MaxHealth=200）500px 范围内 AI 士兵总攻击力 > 300，且距 AI 最近城池距离合理的
- **THEN** AI 选出足够数量的士兵，将其 target 设为该中立城池 Entity

#### Scenario: Insufficient force near neutral city
- **WHEN** 中立城池 500px 范围内 AI 兵力不足
- **THEN** AI 不发起扩张，等待更多士兵产出

### Requirement: AI attack evaluation
AI SHALL 评估最近敌方城池。若敌方城等级 ≤ 己方最高等级，且敌方城 500px 范围内 AI 兵力 > 敌方兵力 × 1.3，则派兵进攻。

#### Scenario: Superior force attacks enemy city
- **WHEN** 玩家城池 Lv.2 500px 范围内 AI 兵力（30 人）> 玩家兵力（20 人）× 1.3
- **THEN** AI 将范围内 AI 士兵的 target 设为玩家城池 Entity

### Requirement: AI upgrade evaluation
AI SHALL 在己方城池人口富余（population > MaxPopulation × 0.6）时，将多余士兵派回城池完成升级。

#### Scenario: Surplus population triggers upgrade
- **WHEN** 敌方城池 population 38/45（> 0.6 × 45 = 27）
- **THEN** AI 将部分（约 38 - 27 = 11）该城产出士兵的 target 设回该城 Entity，使士兵进城贡献升级经验

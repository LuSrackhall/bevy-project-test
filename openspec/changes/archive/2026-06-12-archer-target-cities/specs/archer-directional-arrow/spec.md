## MODIFIED Requirements

### Requirement: 战斗状态互斥

弓兵 SHALL 在攻击冷却恢复时搜索射程内的目标。目标优先级 SHALL 为：先搜索最近敌方兵种，若无敌方兵种则搜索最近敌方城池。有目标时进入 Fighting 状态并发射箭矢，SHALL NOT 在 Fighting 状态下移动。射程内既无兵种也无城池时 SHALL 恢复 Moving 状态。

#### Scenario: 有敌人时射击

- **WHEN** 弓兵攻击冷却为 0 且射程内存在敌方单位
- **THEN** 弓兵发射箭矢瞄准该单位，重置冷却，保持 Fighting 状态

#### Scenario: 无兵种有城池时射击

- **WHEN** 弓兵射程内无敌方兵种但存在敌方城池
- **THEN** 弓兵发射箭矢瞄准最近敌方城池中心，重置冷却，保持 Fighting 状态

#### Scenario: 无目标时恢复移动

- **WHEN** 弓兵 Fighting 状态下，射程内既无兵种也无敌方城池
- **THEN** 弓兵状态变为 Moving，恢复移动

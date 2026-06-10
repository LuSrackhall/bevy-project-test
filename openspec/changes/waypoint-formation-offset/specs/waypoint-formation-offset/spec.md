## ADDED Requirements

### Requirement: waypoint 目标偏移

士兵向 waypoint 移动时 SHALL 将目标位置加上基于 `UnitId` 的确定性偏移。偏移为 32×32 网格，间距 8 内部单位。同一士兵每次到达同一偏移位置。

#### Scenario: 10 个兵右键移动到同一 waypoint

- **WHEN** 10 个兵被命令移动到 waypoint W
- **THEN** 每个兵停在不同位置（W 周围 ±128 单位内的网格点），不重叠

#### Scenario: 同一兵两次到达同一 waypoint

- **WHEN** 兵 A 两次被命令移动到 waypoint W
- **THEN** 两次停在完全相同的偏移位置（基于 UnitId 确定）

### Requirement: 城池目标偏移

士兵以敌方城池为攻击目标时 SHALL 对城池坐标应用 personal_offset。弓兵在城池周围自动分散射击，不全部堆在同一坐标。

#### Scenario: 弓兵攻城自动分散

- **WHEN** 多弓兵以同一城池为目标
- **THEN** 每兵瞄准城池中心 + personal_offset 方向，自然分散在城池周围

### Requirement: 敌方兵种目标不应用偏移

士兵以敌方兵种为目标时 SHALL NOT 应用 personal_offset。近战成环由攻击距离机制负责，分离力辅助散开。

#### Scenario: 近战围攻不变

- **WHEN** 多近战兵以同一敌方兵种为目标
- **THEN** 行为与现有逻辑一致（攻击范围内停步 + 分离力）

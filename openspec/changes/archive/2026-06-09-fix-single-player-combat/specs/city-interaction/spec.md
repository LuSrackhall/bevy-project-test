## ADDED Requirements

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

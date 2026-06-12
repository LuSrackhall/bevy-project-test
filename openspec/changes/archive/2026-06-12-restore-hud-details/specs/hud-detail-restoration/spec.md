## MODIFIED Requirements

### Requirement: Soldier info in bottom panel
底部面板在选中士兵时 SHALL 显示完整的单位信息，包括等级、血量（文本+填充条）、经验（文本+填充条）、攻击力、速度、特殊技能。

#### Scenario: Select single soldier
- **WHEN** 玩家选中一个士兵（HP 45/80, Lv.2, EXP 30/100, ATK 15, SPD 3.0）
- **THEN** 底部面板显示：等级、HP 45/80（绿色填充条 56%）、ATK 15 SPD 3.0、EXP 30/100（紫色填充条 30%）

#### Scenario: Select multiple soldiers
- **WHEN** 玩家选中多个士兵
- **THEN** 底部面板显示汇总信息（总 HP、平均 ATK、兵种分布）

### Requirement: City info in bottom panel
选中己方城池后，底部面板 SHALL 显示城池的完整信息，包括等级、血量（文本+填充条）、经验、人口、训练类型。

#### Scenario: Select player city
- **WHEN** 玩家选中一座己方城池（Lv.2, HP 150/200, EXP 50/100, 人口 15/45）
- **THEN** 底部面板显示：`[城池] Lv.2`、HP 150/200（绿色填充条 75%）、兵 15/45、经验 50/100

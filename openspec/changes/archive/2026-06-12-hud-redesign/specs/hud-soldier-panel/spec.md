## ADDED Requirements

### Requirement: 单选士兵属性展示

选中单个士兵时，左信息面板 SHALL 展示：兵种名、等级、HP 条（含数值）、ATK、SPD、RNG（射程）、碰撞半径（RAD）、EXP 条（含数值）、特殊效果文本、所属城池。

#### Scenario: 选中民兵显示完整属性

- **WHEN** 玩家单击选中一个 Lv.3 民兵
- **THEN** 左面板显示 "民兵 Lv.3"，HP 条显示 当前/最大 值，ATK/SPD/RNG/RAD 显示具体数值，EXP 条显示 当前/升级所需，特殊效果 "无"，所属城池显示城池名

### Requirement: 多选士兵聚合展示

选中多个士兵时，左面板 SHALL 按兵种分组计数并显示总 HP 和平均 ATK。各兵种组用不同标记区分。

#### Scenario: 选中混合兵种

- **WHEN** 玩家框选 3 民兵 + 2 弓兵 + 1 骑兵
- **THEN** 面板显示 "选中 6 个单位"，按兵种列出 "3 民兵 / 2 弓兵 / 1 骑兵"，总 HP 和均 ATK

## ADDED Requirements

### Requirement: 兵种图鉴悬停显示

城池面板或任何兵种按钮被悬停（`Interaction::Hovered`）时，右区图鉴 SHALL 显示该兵种的完整属性 + 特殊效果 + 描述文本。图鉴数据 SHALL 存储于 render_view 层的资源（非 simulation 层）。

#### Scenario: 悬停骑兵按钮

- **WHEN** 玩家鼠标悬停在城池面板的骑兵按钮上
- **THEN** 右区兵种图鉴显示：骑兵 HP/ATK/SPD/RNG/RAD、特殊效果 "闪避+无畏"、描述 "重骑兵，高速冲锋陷阵。受伤时可闪避近战攻击并激活无畏状态。"

#### Scenario: 无悬停时占位

- **WHEN** 没有兵种按钮被悬停
- **THEN** 右区兵种图鉴显示 "悬停兵种按钮查看详情"

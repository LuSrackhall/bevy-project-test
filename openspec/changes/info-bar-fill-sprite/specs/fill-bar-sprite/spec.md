## MODIFIED Requirements

### Requirement: Health bar fill displays correct ratio
血量条填充矩形 SHALL 使用 Bevy 原生 `Sprite` 组件渲染，按 `Health.current / Health.max` 的比例正确显示绿色填充宽度，左边缘对齐背景条左边缘。

#### Scenario: Full health
- **WHEN** 单位 HP 为 100/100
- **THEN** 绿色 `Sprite` 填充宽度等于背景条宽度（40px），左边缘对齐红色背景条左边缘

#### Scenario: Half health
- **WHEN** 单位 HP 为 50/100
- **THEN** 绿色 `Sprite` 填充宽度为背景条宽度的 50%（20px），左边缘对齐红色背景条左边缘

#### Scenario: Health changes dynamically
- **WHEN** 单位 HP 从 100/100 变为 50/100
- **THEN** 绿色 `Sprite` 填充宽度从 40px 变为 20px，无需 despawn/respawn 实体

### Requirement: Experience bar fill displays correct ratio
经验条填充矩形 SHALL 使用 Bevy 原生 `Sprite` 组件渲染，按 `Level.exp / EXP_MAX` 的比例正确显示紫色填充宽度，左边缘对齐背景条左边缘。

#### Scenario: Half experience
- **WHEN** 单位 EXP 为 50/100
- **THEN** 紫色 `Sprite` 填充宽度为背景条宽度的 50%（20px），左边缘对齐蓝色背景条左边缘

#### Scenario: Zero experience
- **WHEN** 单位 EXP 为 0/100
- **THEN** 紫色 `Sprite` 填充 `custom_size` 宽度为 0（不可见）

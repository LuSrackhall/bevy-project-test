## MODIFIED Requirements

### Requirement: Health bar fill displays correct ratio
血量条填充矩形 SHALL 按 `Health.current / Health.max` 的比例正确显示绿色填充宽度，左边缘对齐背景条左边缘。

#### Scenario: Full health
- **WHEN** 单位 HP 为 100/100
- **THEN** 绿色填充矩形宽度等于背景条宽度（40px），左边缘对齐红色背景条左边缘

#### Scenario: Half health
- **WHEN** 单位 HP 为 50/100
- **THEN** 绿色填充矩形宽度为背景条宽度的 50%（20px），左边缘对齐红色背景条左边缘

#### Scenario: Low health
- **WHEN** 单位 HP 为 10/100
- **THEN** 绿色填充矩形宽度为背景条宽度的 10%（4px），左边缘对齐红色背景条左边缘

### Requirement: Experience bar fill displays correct ratio
经验条填充矩形 SHALL 按 `Level.exp / EXP_MAX` 的比例正确显示紫色填充宽度，左边缘对齐背景条左边缘。

#### Scenario: Half experience
- **WHEN** 单位 EXP 为 50/100
- **THEN** 紫色填充矩形宽度为背景条宽度的 50%（20px），左边缘对齐蓝色背景条左边缘

#### Scenario: Zero experience
- **WHEN** 单位 EXP 为 0/100
- **THEN** 紫色填充矩形宽度为 0（不可见）

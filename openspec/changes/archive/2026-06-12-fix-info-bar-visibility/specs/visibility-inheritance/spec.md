## MODIFIED Requirements

### Requirement: Info bar children inherit parent visibility
信息条子实体（文字、背景条、填充条）SHALL 使用 `Visibility::Inherited`，由根实体统一控制可见性。子实体不得使用 `Visibility::Hidden`。

#### Scenario: Bar created in hidden mode then shown
- **WHEN** Smart 模式下 `should_show = false`，信息条被创建（根实体 `Hidden`）
- **WHEN** 随后切换到 Always 模式（`should_show = true`）
- **THEN** `update_bar` 将根实体设为 `Inherited` 后，所有子实体（文字、背景条、填充条）立即可见

#### Scenario: City bar displays in Always mode
- **WHEN** 城池在游戏开始时创建（Smart 模式，满血，`should_show = false`）
- **WHEN** 随后切换到 Always 模式
- **THEN** 城池血条（背景、填充、文字）全部正确显示

#### Scenario: Existing soldier bar shows after mode switch
- **WHEN** 一个满血士兵在 Smart 模式下创建（无血条显示）
- **WHEN** 切换到 Always 模式
- **THEN** 该士兵头顶立即显示完整的血条、经验条和文字

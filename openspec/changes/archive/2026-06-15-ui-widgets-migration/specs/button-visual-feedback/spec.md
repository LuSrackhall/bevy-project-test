## ADDED Requirements

### Requirement: ButtonTheme 组件

系统 SHALL 提供 `ButtonTheme` 组件，允许不同按钮定义不同的视觉样式。

#### Scenario: 默认主题

- **WHEN** 按钮没有显式设置 `ButtonTheme`
- **THEN** 系统 SHALL 使用默认主题颜色（normal/hovered/pressed）

#### Scenario: 自定义主题

- **WHEN** 按钮设置了自定义 `ButtonTheme`（如红色攻击按钮）
- **THEN** 系统 SHALL 使用自定义颜色

### Requirement: 集中式 button_style_system

系统 SHALL 提供集中的 `button_style_system`，根据交互状态驱动按钮视觉反馈。

#### Scenario: 悬停高亮

- **WHEN** 鼠标悬停在按钮上
- **THEN** 按钮背景色 SHALL 变为 `ButtonTheme.hovered` 颜色

#### Scenario: 按下效果

- **WHEN** 鼠标按下按钮
- **THEN** 按钮背景色 SHALL 变为 `ButtonTheme.pressed` 颜色

#### Scenario: 恢复正常

- **WHEN** 鼠标离开按钮且未按下
- **THEN** 按钮背景色 SHALL 恢复为 `ButtonTheme.normal` 颜色

### Requirement: 视觉反馈使用 PickingInteraction

`button_style_system` SHALL 使用 `PickingInteraction` 驱动视觉状态，不使用 `Interaction`。

#### Scenario: PickingInteraction 驱动样式

- **WHEN** 按钮的 `PickingInteraction` 状态变化
- **THEN** `button_style_system` SHALL 更新 `BackgroundColor`

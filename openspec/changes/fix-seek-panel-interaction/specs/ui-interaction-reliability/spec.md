## ADDED Requirements

### Requirement: 弹出层始终渲染

下拉弹出层节点 SHALL 始终保持 `Display::Flex` 和 `Visibility::Visible`。关闭状态 SHALL 通过将 `Node.left` 设为 `Val::Px(-9999.0)` 将节点移出可视区域。打开状态 SHALL 将 `Node.left` 恢复为 `Val::Px(0.0)`。

#### Scenario: 弹出层关闭时不可见

- **WHEN** `SeekPanelState.dropdown_open == false`
- **THEN** 弹出层 `Node.left == Val::Px(-9999.0)`，节点在屏幕外不可见
- **AND** 弹出层子节点（选项按钮）的 `Interaction` 组件正常存在

#### Scenario: 弹出层打开时可见

- **WHEN** `SeekPanelState.dropdown_open == true`
- **THEN** 弹出层 `Node.left == Val::Px(0.0)`，节点在正常位置显示
- **AND** 选项按钮可正常响应鼠标交互

### Requirement: 交互检测不依赖 Changed<Interaction>

所有 seek panel 交互系统（dropdown、input、issue）SHALL 使用 `Query<&Interaction, With<Marker>>` 而非 `Query<..., Changed<Interaction>>`。每帧 SHALL 检查 `Interaction::Pressed` 状态并配合 `mouse.just_pressed(MouseButton::Left)` 做防抖。

#### Scenario: 下拉选项点击选中

- **WHEN** 弹出层可见，用户点击"步兵"选项
- **THEN** `SeekPanelState.scope` 更新为 `ByType(Infantry)`
- **AND** 弹出层关闭

#### Scenario: 持续按住不重复触发

- **WHEN** 用户按住鼠标按钮超过 1 帧
- **THEN** 交互系统仅在按下第一帧响应，后续帧不重复触发

### Requirement: 输入框编辑防重入

范围输入框 SHALL 使用 `mouse.just_pressed(MouseButton::Left)` + `!state.editing` 双重防护确保仅在点击瞬间进入编辑模式一次。

#### Scenario: 点击输入框进入编辑

- **WHEN** 输入框未处于编辑态，用户点击输入框
- **THEN** `SeekPanelState.editing` 设为 `true`，`input_buffer` 初始化为当前范围值

#### Scenario: 编辑态中点击输入框不重入

- **WHEN** 输入框已处于编辑态，用户再次点击输入框
- **THEN** 编辑态保持不变，`input_buffer` 不被重置

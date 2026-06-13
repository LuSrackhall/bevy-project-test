## ADDED Requirements

### Requirement: 弹出层使用 Display 切换

下拉弹出层 SHALL 使用 `Display::None` 隐藏、`Display::Flex` 显示。SHALL NOT 使用位置偏移隐藏。

#### Scenario: 弹出层正确隐藏和显示

- **WHEN** `dropdown_open == false`，弹出层 `display == Display::None`
- **WHEN** `dropdown_open == true`，弹出层 `display == Display::Flex`
- **THEN** 子节点 `Interaction` 在显示帧正确初始化

### Requirement: Text 实体 ID 正确存储

`HudTexts.seek_scope_text` 和 `HudTexts.seek_range_text` SHALL 存储 Text 子实体 ID，NOT Button 父实体 ID。

#### Scenario: 下拉触发器文字更新

- **WHEN** 选择 scope 为"步兵"
- **THEN** 触发器按钮文字更新为"步兵 ▼"

#### Scenario: 输入框文字更新

- **WHEN** 范围值为 30
- **THEN** 输入框显示"30"，编辑态显示"30▌"

### Requirement: 选择系统跳过 UI 点击

`selection_click_system` SHALL 检测光标是否在 UI 元素上。若是，SHALL 跳过所有选择逻辑（不清空选区、不选中单位）。

#### Scenario: 点击 UI 不清空选区

- **WHEN** 选中了 3 个单位，用户点击索敌面板的输入框
- **THEN** 选区保持不变（3 个单位仍选中）

### Requirement: 编辑态不被模式切换重置

`seek_panel_mode_system` SHALL 在 `SeekPanelState.editing == true` 时不执行模式切换（不重置 range_value、input_buffer、editing）。

#### Scenario: 编辑中选区变化不中断编辑

- **WHEN** 用户正在编辑范围输入框，选区被清空
- **THEN** 编辑态保持，输入框继续接收键盘输入

### Requirement: 系统执行顺序

HUD 交互系统（seek_panel_*、toolbar_button）SHALL 在选择系统（selection_click_system、drag_select_system）之前运行。

#### Scenario: HUD 系统先于选择系统

- **WHEN** 同一帧内用户点击了 UI 按钮
- **THEN** HUD 系统先处理交互，选择系统检测到 UI 交互后跳过

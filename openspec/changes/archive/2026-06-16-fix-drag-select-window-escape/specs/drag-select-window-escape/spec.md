## ADDED Requirements

### Requirement: 框选过程中鼠标移出窗口时保持状态

当鼠标在框选过程中移出窗口时，系统 SHALL 保持 `drag_current` 不变，继续执行后续逻辑。

#### Scenario: 鼠标移出窗口后框选矩形保持

- **WHEN** 玩家按住左键拖拽形成框选矩形，然后鼠标移出窗口
- **THEN** 框选矩形 SHALL 保持在最后的窗口内位置，不卡住也不消失

#### Scenario: 鼠标在窗口外松开完成框选

- **WHEN** 玩家按住左键拖拽形成框选矩形，鼠标移出窗口后松开左键
- **THEN** 框选 SHALL 完成，根据起点和最后的窗口内位置选中范围内的单位

#### Scenario: 鼠标在窗口内松开行为不变

- **WHEN** 玩家按住左键拖拽形成框选矩形，鼠标在窗口内松开左键
- **THEN** 框选 SHALL 正常完成，行为与修改前一致

### Requirement: 改用全局鼠标状态检测松开

系统 SHALL 使用 `!mouse.pressed(MouseButton::Left)` 检测鼠标松开，替代 `mouse.just_released(MouseButton::Left)`。

#### Scenario: 窗口外松开检测

- **WHEN** 玩家在窗口外松开左键
- **THEN** `!pressed` SHALL 返回 true，框选完成

## ADDED Requirements

### Requirement: Click to select friendly soldiers
玩家在 Playing 状态下点击己方士兵时，SHALL 选中该士兵并显示选中标记。点击空地且无拖拽时取消所有选中。

#### Scenario: Click single soldier
- **WHEN** 玩家左键（桌面）或单指（移动端）点击己方士兵（世界坐标命中士兵视觉形状）
- **THEN** 该士兵被选中，附加 SelectionIndicator（半透明绿色圆环），取消之前的所有选中

#### Scenario: Click empty ground with existing selection
- **WHEN** 玩家点击地图空白区域且当前有选中的士兵
- **THEN** 取消所有选中，移除所有 SelectionIndicator

### Requirement: Drag-select soldiers in rectangle mode
玩家左键拖拽（桌面）或单指拖拽（移动端）时，SHALL 在矩形选区模式下框选所有己方士兵。拖拽起点命中己方单位时进入框选模式（不触发摄像机平移）。

#### Scenario: Rect drag over 3 friendly soldiers
- **WHEN** 玩家在矩形模式下拖拽出一个包含 3 个己方士兵的矩形区域并释放
- **THEN** 这 3 个士兵被选中，各自附加 SelectionIndicator

#### Scenario: Drag starting on friendly soldier
- **WHEN** 玩家拖拽起点命中己方士兵
- **THEN** 进入框选模式（start→end 形成选区），摄像机不平移

#### Scenario: Drag starting on empty ground (desktop)
- **WHEN** 玩家拖拽起点是空地（桌面端）
- **THEN** 摄像机平移（不进入框选模式，因为桌面端摄像机平移用中键/右键，此 case 仅移动端触发）

#### Scenario: Drag starting on empty ground (mobile)
- **WHEN** 玩家拖拽起点是空地（移动端）
- **THEN** 摄像机平移（移动端与框选的冲突通过起点命中检测解决）

### Requirement: Circle selection mode
框选模式切换为圆形时，SHALL 以拖拽起点为圆心、拖拽距离为半径，框选所有己方士兵。

#### Scenario: Circle select
- **WHEN** 框选模式为 Circle，玩家拖拽半径为 100px
- **THEN** 距离拖拽起点 ≤ 100px 的所有己方士兵被选中

### Requirement: Right-click commands selected soldiers to move/attack
右键点击（桌面）或轻点空地/敌方/敌方城池（移动端）时，SHALL 将选中士兵的 target 设为目标 Entity。空地时生成 Waypoint Entity。

#### Scenario: Attack enemy soldier
- **WHEN** 玩家选中己方士兵，右键点击敌方士兵
- **THEN** 所有选中士兵的 target 设为该敌方士兵 Entity

#### Scenario: Attack enemy city
- **WHEN** 玩家选中己方士兵，右键点击敌方城池
- **THEN** 所有选中士兵的 target 设为该敌方城池 Entity

#### Scenario: Move to empty ground
- **WHEN** 玩家选中己方士兵，右键点击空地
- **THEN** 生成隐形 Waypoint Entity，所有选中士兵的 target 设为 Waypoint Entity。士兵到达 Waypoint 10px 范围内后 target 清空，Waypoint despawn

### Requirement: Select all shortcut
玩家按下 Ctrl+A（桌面）时，SHALL 选中所有己方士兵。

#### Scenario: Ctrl+A selects all
- **WHEN** 玩家按下 Ctrl+A
- **THEN** 所有 faction == Player 的士兵被选中，附加 SelectionIndicator

### Requirement: Escape deselects all
玩家按下 Esc（桌面）时，SHALL 取消所有选中。

#### Scenario: Esc clears selection
- **WHEN** 玩家按下 Esc 且有选中士兵
- **THEN** 所有选中取消，SelectionIndicator 移除。Playing 状态下 Esc 仍触发暂停（保持现有功能，但取消选中优先级更高）

### Requirement: Selection mode toggle
底部工具栏的圆形/方形框选按钮 SHALL 切换 SelectionState.selection_mode。

#### Scenario: Switch to circle mode
- **WHEN** 玩家点击 "○框选" 按钮
- **THEN** SelectionState.selection_mode 切换为 Circle

## MODIFIED Requirements

### Requirement: 主菜单系统
主菜单 SHALL 提供 4 个地图大小按钮（小/中/大/巨大），点击后设 `NeedsGameReset::NewGame(MapSize)` 并进入 `Playing`。

#### Scenario: 选择 Huge 地图
- **WHEN** 用户点击"巨大"
- **THEN** `NeedsGameReset` 设为 `NewGame(MapSize::Huge)`，`GameState` 切换到 `Playing`

### Requirement: 相机系统
镜头缩放范围 SHALL 动态计算。拖拽 SHALL 仅中键。镜头位置 SHALL 限制在地图边界内。

#### Scenario: 动态缩放
- **WHEN** 地图 8000x8000，窗口 1280x720
- **THEN** 缩放范围 [0.15, 11.11]

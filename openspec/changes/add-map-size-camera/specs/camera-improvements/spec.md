# camera-improvements

## Purpose

镜头操作优化：动态缩放范围、中键拖拽自适应速度、边界限制。

## ADDED Requirements

### Requirement: 动态缩放范围
镜头缩放范围 SHALL 根据地图大小和窗口尺寸动态计算。`max_scale = max(map_width / window_width, map_height / window_height, 1.0)`。`min_scale = 0.15`。使用当前窗口尺寸每帧计算。

#### Scenario: Small 地图缩放
- **WHEN** 地图 2000x2000，窗口 1280x720
- **THEN** `max_scale = max(2000/1280, 2000/720, 1.0) = 2.78`

#### Scenario: Huge 地图缩放
- **WHEN** 地图 8000x8000，窗口 1280x720
- **THEN** `max_scale = max(8000/1280, 8000/720, 1.0) = 11.11`

#### Scenario: 窗口 resize
- **WHEN** 窗口从 1280x720 resize 到 640x360
- **THEN** `max_scale` 自动重新计算为更大值

### Requirement: 中键拖拽
镜头拖拽 SHALL 仅响应中键鼠标。右键 SHALL 专用于指令。拖拽速度 = `delta * ortho.scale`（线性自适应）。

#### Scenario: 中键拖拽
- **WHEN** 按住中键移动鼠标
- **THEN** 镜头随鼠标移动，速度与缩放比例成正比

#### Scenario: 右键不拖拽
- **WHEN** 按住右键移动鼠标
- **THEN** 镜头不移动（右键用于指令）

### Requirement: 镜头边界限制
镜头位置 SHALL 限制在地图边界内。padding = `map_size * 0.05`，最小 100 单位。Clamp after zoom。

#### Scenario: Small 地图边界
- **WHEN** 地图 2000x2000，padding = 100
- **THEN** 镜头 x ∈ [-100, 2100]，y ∈ [-100, 2100]

#### Scenario: Huge 地图边界
- **WHEN** 地图 8000x8000，padding = 400
- **THEN** 镜头 x ∈ [-400, 8400]，y ∈ [-400, 8400]

### Requirement: MapBounds 桥接
bevy_adapter SHALL 提供 `MapBounds { width: f32, height: f32 }` 资源，从 `MapGenConfig` 纯翻译。render_view 读取 `MapBounds` 计算缩放限制和边界。

#### Scenario: MapBounds 创建
- **WHEN** `reset_game_system` 完成，`MapGenConfig` 为 3500x3500
- **THEN** `MapBounds { width: 3500.0, height: 3500.0 }` 被创建

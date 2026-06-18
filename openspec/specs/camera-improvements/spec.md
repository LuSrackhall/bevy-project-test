# camera-improvements

## Purpose

镜头操作优化：动态缩放范围、中键拖拽自适应速度、边界限制。

## Requirements

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

### Requirement: 边缘滚动
鼠标靠近屏幕边缘时镜头 SHALL 自动滚动。边缘区域 30px，基础速度 800 units/s，随缩放比例自适应。

#### Scenario: 左边缘滚动
- **WHEN** 鼠标在屏幕左边缘 30px 内
- **THEN** 镜头自动向左移动

#### Scenario: 缩放自适应
- **WHEN** 缩放比例为 3.0
- **THEN** 滚动速度为 800 * 3.0 = 2400 units/s

### Requirement: 光标居中缩放
缩放 SHALL 保持光标下方的世界点不动（zoom toward cursor）。

#### Scenario: 缩放时世界点固定
- **WHEN** 光标在某个城池上，向上滚动缩放
- **THEN** 该城池仍在光标下方，其他区域相对移动

### Requirement: 边界墙
边界墙 SHALL 为 2 倍地图大小（如 2000x2000 → 边界 -500~1500），红色线条可视化，单位移动被 clamp 在边界内。最大缩放 = 6 倍地图大小。

#### Scenario: 单位到达边界
- **WHEN** 士兵移动到边界墙位置
- **THEN** 士兵停在边界墙处，不越过

#### Scenario: 边界墙可见
- **WHEN** 缩放到最大范围
- **THEN** 四条红色线条显示边界墙位置

### Requirement: 默认全屏
游戏 SHALL 以无边框全屏模式启动。

#### Scenario: 启动全屏
- **WHEN** 游戏启动
- **THEN** 窗口为 BorderlessFullscreen 模式

### Requirement: MapBounds 桥接
bevy_adapter SHALL 提供 `MapBounds { width: f32, height: f32 }` 资源，从 `MapGenConfig` 纯翻译。render_view 读取 `MapBounds` 计算缩放限制和边界。

#### Scenario: MapBounds 创建
- **WHEN** `reset_game_system` 完成，`MapGenConfig` 为 3500x3500
- **THEN** `MapBounds { width: 3500.0, height: 3500.0 }` 被创建

## Why

当前游戏地图固定为 2000x2000，没有规模选择。镜头缩放范围固定（0.2-3.0），大地图无法看全貌；右键拖拽与指令冲突；镜头可拖出边界。需要地图规模预设和镜头操作优化。

## What Changes

- **BREAKING**: `NeedsGameReset` 从 `bool` 改为枚举（`None`/`SameSize`/`NewGame(MapSize)`）
- 新增 `MapSize` 枚举（Small/Medium/Large/Huge）在 simulation crate
- `content/map.ron` 拆分为 `content/map/{small,medium,large,huge}.ron` 四个预设配置文件
- `generate_map` 签名改为接受 `MapSize` 参数，城池数量改为密度驱动（1 per 250K sq units）
- `neutral_city_ratio` 从 `[f32; 2]` 改为 `[u32; 2]`（宪法合规）
- 新增 `MapBounds { width: f32, height: f32 }` 资源在 bevy_adapter
- 镜头缩放范围动态计算（基于地图大小和窗口尺寸）
- 拖拽改为仅中键，速度随缩放自适应
- 镜头边界限制在地图范围内
- 主菜单增加地图大小选择按钮

## Capabilities

### New Capabilities
- `map-presets`: 地图规模预设系统——MapSize 枚举、密度驱动城池数、配置文件拆分、主菜单选择 UI
- `camera-improvements`: 镜头操作优化——动态缩放、中键拖拽、边界限制、MapBounds 桥接

### Modified Capabilities
- `simulation-crate`: MapSize 枚举定义、generate_map 签名变更、neutral_city_ratio 类型修复
- `bevy-adapter-crate`: MapBounds 资源、NeedsGameReset 枚举变更
- `render-view-crate`: 主菜单增加地图选择、镜头系统重写
- `game-lifecycle`: NeedsGameReset 从 bool 改为枚举

- 边缘滚动（鼠标靠近屏幕边缘自动移动镜头）
- 光标居中缩放（zoom toward cursor）
- 边界墙系统（2 倍地图大小，红色可视化，单位不可穿越）
- 默认无边框全屏启动
- 框选框描边宽度随缩放自适应

## Impact

- `simulation/src/map/mod.rs`: generate_map 改为接受 MapSize
- `simulation/src/map/config.rs`: MapGenConfig 新增 density 字段，neutral_city_ratio 改 u32
- `content/map/*.ron`: 4 个新配置文件
- `bevy_adapter/src/lib.rs`: MapBounds 资源、NeedsGameReset 枚举
- `render_view/src/camera.rs`: 缩放/拖拽/边界逻辑重写
- `render_view/src/lib.rs`: NeedsGameReset 枚举更新
- `render_view/src/ui/menu.rs`: 地图大小选择 UI
- `render_view/src/ui/gameover.rs`、`pause.rs`: NeedsGameReset 用法更新

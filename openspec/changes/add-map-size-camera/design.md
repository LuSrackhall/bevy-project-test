## Context

当前 `generate_map` 使用 `MapGenConfig` 中的固定值（2000x2000, 6-20 cities）。`NeedsGameReset` 是 `bool`。镜头系统使用硬编码缩放范围和 1:1 像素拖拽。`neutral_city_ratio` 使用 `f32` 违反宪法。

所有变更在 simulation（MapSize 定义）、bevy_adapter（MapBounds 桥接）、render_view（镜头/UI）、content（配置文件）中。simulation 层变更最小（新增枚举 + generate_map 参数）。

## Goals / Non-Goals

**Goals:**
- 4 级地图预设，密度驱动城池数
- 动态缩放范围，自适应拖拽速度
- 中键拖拽，右键专用于指令
- 镜头边界限制
- 宪法合规（f32 修复）

**Non-Goals:**
- 不实现地形障碍物、小地图、自定义滑块

## Decisions

### NeedsGameReset 枚举

```rust
#[derive(Resource)]
pub enum NeedsGameReset {
    None,              // 暂停恢复
    SameSize,          // 重开
    NewGame(MapSize),  // 新游戏
}
```

替换所有 `needs_reset.0 = true` 为 `NeedsGameReset::SameSize` 或 `NeedsGameReset::NewGame(MapSize)`。

### MapSize 定义位置

MapSize 在 `simulation/src/map/mod.rs`（纯数据枚举）。generate_map 接受 MapSize，内部查找对应配置。

### 配置文件结构

```
content/map/
  small.ron    → MapGenConfig { width: 2000, height: 2000, city_density: 250000, ... }
  medium.ron   → MapGenConfig { width: 3500, height: 3500, city_density: 250000, ... }
  large.ron    → MapGenConfig { width: 5000, height: 5000, city_density: 250000, ... }
  huge.ron     → MapGenConfig { width: 8000, height: 8000, city_density: 250000, ... }
```

每个文件独立完整，`MapGenConfig` 新增 `city_density: u32` 字段（每 N 平方单位 1 城池）。`neutral_city_ratio` 改为 `[u32; 2]`。

### 城池数量计算

```rust
let area = config.width as u64 * config.height as u64;
let base_count = area / config.city_density as u64;
let variance = base_count * 15 / 100; // ±15%
let city_count = rng.gen_range((base_count - variance) as u32, (base_count + variance) as u32);
```

### MapBounds 桥接

bevy_adapter 在 `reset_game_system` 后读取 `MapGenConfig`，创建 `MapBounds` 资源：
```rust
#[derive(Resource)]
pub struct MapBounds { pub width: f32, pub height: f32 }
```

render_view 的镜头系统读取 `MapBounds` + 窗口尺寸计算缩放和边界。

### 镜头缩放

```rust
let max_scale = (map_bounds.width / window.width())
    .max(map_bounds.height / window.height())
    .max(1.0);
let min_scale = 0.15;
```

每帧使用当前窗口尺寸（不缓存）。

### 镜头拖拽

- 仅中键（移除右键拖拽）
- `translation -= delta * ortho.scale`
- 边界 padding = `map_size * 0.05`，最小 100
- Clamp after zoom

### 主菜单 UI

4 个按钮：小/中/大/巨大。点击后设 `NeedsGameReset::NewGame(MapSize)` + `NextState(Playing)`。

## Risks / Trade-offs

- **[权衡] Huge 200-280 城池** → Bevy ECS 轻松处理，后续可优化
- **[风险] 触控板无中键** → 后续可加 WASD 滚动
- **[风险] 配置文件重复** → 后续可加共享默认值机制

### 边缘滚动

鼠标靠近屏幕边缘 30px 内自动滚动镜头。基础速度 800 units/s，随缩放比例自适应。四方向均可触发。

### 光标居中缩放（Zoom Toward Cursor）

缩放时保持光标下方的世界点不动。实现：根据缩放比例变化调整 camera translation，使 cursor 处的世界坐标不变。

### 边界墙

边界墙 = 2 倍地图大小（如 2000x2000 → 边界 [-500, 1500]）。红色线条可视化。单位移动 clamp 到 WallBounds。镜头拖拽 clamp 到 wall_min/wall_max。

WallBounds 定义在 simulation 层（纯数据），generate_map 时计算并插入。MapBounds 在 bevy_adapter 层桥接 wall_min/max 字段给 render_view。

最大缩放 = 6 倍地图大小（确保边界墙完全可见）。

### 默认全屏

`BorderlessFullscreen(MonitorSelection::Primary)` 无边框全屏启动。避免鼠标移出窗口。

### 缩放参数

- 步长：0.6%（乘法，`*= 1.0 - delta * 0.006`）
- 最小缩放：0.15
- 最大缩放：`map_dim * 6.0 / min(window_w, window_h)`

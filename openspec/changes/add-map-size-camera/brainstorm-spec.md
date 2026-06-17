## Context

当前游戏地图固定为 2000x2000，6-20 座城池，配置在 `content/map.ron` 中。没有地图规模选择功能，也没有地形障碍物。镜头系统存在以下问题：
- 缩放范围固定 0.2-3.0，大地图无法看全貌
- 拖拽速度与缩放比例脱钩
- 右键拖拽与右键指令冲突
- 镜头可以拖出地图边界

用户需求：提供 4 级地图规模预设（小/中/大/巨大），优化镜头操作体验。

## Goals / Non-Goals

**Goals:**
- 提供 4 级地图规模预设：Small(2000)、Medium(3500)、Large(5000)、Huge(8000)
- 城池数量根据地图面积密度自动计算（1 city per 250,000 sq units ±15%）
- 缩放范围根据地图大小和窗口尺寸动态计算
- 拖拽速度随缩放比例自适应
- 拖拽改为仅中键（右键专用于指令）
- 镜头边界限制在地图范围内
- 修复 `neutral_city_ratio` 的 f32 违规（改用 u32 百分比）

**Non-Goals:**
- 不实现地形障碍物（后续变更）
- 不实现小地图（后续独立变更）
- 不实现自定义滑块（后续在预设基础上扩展）
- 不改变地图生成算法的核心逻辑（只改配置和参数）

## Decisions

### 决策 1：NeedsGameReset 枚举替代 bool

**选择**：`NeedsGameReset` 从 `bool` 改为显式枚举。

```rust
#[derive(Resource)]
pub enum NeedsGameReset {
    None,              // 暂停恢复，不重置
    SameSize,          // 重开当前大小的地图
    NewGame(MapSize),  // 用指定大小开始新游戏
}
```

**理由**：消除了 `Option<MapSize>` 中 `None` 的歧义（"不重置"还是"用默认大小重置"）。三个变体覆盖所有生命周期转换。

### 决策 2：MapSize 枚举 + 密度驱动城池数

**选择**：在 simulation crate 定义 `MapSize` 枚举（纯数据），配置文件按预设拆分。

```rust
pub enum MapSize { Small, Medium, Large, Huge }
```

城池数量用密度常量计算：`1 city per 250,000 sq units`，±15% 随机范围。使用 simulation 的 `DeterministicRng` 保证确定性。

| 预设 | 尺寸 | 面积 | 城池数 |
|------|------|------|--------|
| Small | 2000x2000 | 4M | 12-20 |
| Medium | 3500x3500 | 12.25M | 40-55 |
| Large | 5000x5000 | 25M | 80-110 |
| Huge | 8000x8000 | 64M | 200-280 |

**理由**：密度驱动避免了固定范围的重叠问题（如 Small max=20 > Medium min=12）。

### 决策 3：配置文件拆分

**选择**：`content/map.ron` 拆分为 `content/map/small.ron`、`content/map/medium.ron`、`content/map/large.ron`、`content/map/huge.ron`。

**理由**：每预设独立文件，自包含，支持热重载单个预设。

### 决策 4：f32 违规修复

**选择**：`neutral_city_ratio` 从 `[f32; 2]` 改为 `[u32; 2]`（百分比，如 `[30, 50]`）。仿真层用 `count * ratio / 100` 整数运算。

**理由**：符合宪法第 2.2 节——simulation 中禁止 f32。

### 决策 5：MapBounds 桥接

**选择**：`MapBounds { width: f32, height: f32 }` 定义在 `bevy_adapter`，从 `MapGenConfig` 纯翻译。镜头逻辑（缩放/边界/clamp）全部在 `render_view`。

```
simulation: MapGenConfig{width: u32, height: u32}
  ↓
bevy_adapter: MapBounds{width: f32, height: f32}（纯数据翻译）
  ↓
render_view: 读取 MapBounds + 窗口尺寸 → 计算缩放限制和边界
```

**理由**：bevy_adapter 负责数据翻译，render_view 负责渲染决策，符合单向依赖拓扑。

### 决策 6：动态缩放范围

**选择**：缩放范围根据地图大小和窗口尺寸动态计算。

```rust
let max_scale = (map_bounds.width / window.width())
    .max(map_bounds.height / window.height())
    .max(1.0);  // floor: 至少 1:1
let min_scale = 0.15;
```

使用当前窗口尺寸每帧计算（不缓存），适配窗口 resize。

**理由**：Bevy 的 orthographic projection 中 scale 均匀应用于两轴，visible_width = scale * window_width，visible_height = scale * window_height。公式取两个轴的约束最大值，确保地图始终在视野内。

### 决策 7：拖拽优化

**选择**：
- 仅中键拖拽（移除右键拖拽，右键专用于指令）
- 拖拽速度 = `delta * ortho.scale`（线性自适应）
- 边界 padding = `map_size * 0.05`，最小 100
- Clamp after zoom（先缩放后限制）

**理由**：符合 RTS 惯例（StarCraft/AoE/WC3 均为中键拖拽、右键指令）。线性速度保持"所见即所移"的直觉。

### 决策 8：主菜单地图选择 UI

**选择**：主菜单增加 4 个地图大小按钮（小/中/大/巨大），点击后设 `NeedsGameReset(NewGame(MapSize))` 并进入游戏。

**理由**：预设按钮比滑块更简洁，点击即开始。

## Risks / Trade-offs

- **[权衡] Huge 地图 200-280 城池** → Bevy ECS 可以轻松处理（数千实体无压力），但每个城池产兵后实体数可能达数千。后续如遇性能问题可优化。
- **[风险] 缩放公式假设 ortho scale 均匀** → 已通过 reviewer 确认 Bevy 0.18 的 OrthographicProjection.scale 确实均匀应用于两轴。
- **[风险] 中键拖拽在触控板上不可用** → 后续可添加键盘+WASD 滚动作为补充。本次不实现。
- **[风险] 配置文件拆分导致大量重复** → 每预设文件结构相同但数值不同。后续可考虑共享默认值 + 覆盖机制。

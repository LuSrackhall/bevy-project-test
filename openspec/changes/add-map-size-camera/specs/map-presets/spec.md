# map-presets

## Purpose

地图规模预设系统：MapSize 枚举、密度驱动城池数、配置文件拆分、主菜单选择 UI。

## ADDED Requirements

### Requirement: MapSize 枚举
simulation crate SHALL 定义 `MapSize` 枚举（Small/Medium/Large/Huge），用于指定地图规模。`generate_map` SHALL 接受 `MapSize` 参数，加载对应配置文件。

#### Scenario: 选择 Small 地图
- **WHEN** 调用 `generate_map(world, MapSize::Small)`
- **THEN** 加载 `content/map/small.ron` 配置，生成 2000x2000 地图

#### Scenario: 选择 Huge 地图
- **WHEN** 调用 `generate_map(world, MapSize::Huge)`
- **THEN** 加载 `content/map/huge.ron` 配置，生成 8000x8000 地图

### Requirement: 密度驱动城池数量
城池数量 SHALL 根据地图面积和密度常量计算，允许 ±15% 随机波动。密度常量在配置文件中定义（默认 250000 sq units per city）。

#### Scenario: Small 地图城池数
- **WHEN** 面积 4,000,000，密度 250,000
- **THEN** 基础城池数 16，实际 12-20（±15%）

#### Scenario: Huge 地图城池数
- **WHEN** 面积 64,000,000，密度 250,000
- **THEN** 基础城池数 256，实际 200-280（±15%）

#### Scenario: 确定性
- **WHEN** 相同种子和 MapSize
- **THEN** 城池数量和位置完全相同

### Requirement: 配置文件拆分
地图配置 SHALL 拆分为 `content/map/{small,medium,large,huge}.ron` 四个独立文件。每个文件包含完整的 `MapGenConfig`。

#### Scenario: 加载 Medium 配置
- **WHEN** `MapSize::Medium`
- **THEN** 从 `content/map/medium.ron` 加载配置

### Requirement: neutral_city_ratio 类型修复
`neutral_city_ratio` SHALL 从 `[f32; 2]` 改为 `[u32; 2]`（百分比，如 `[30, 50]`）。仿真层用整数运算 `count * ratio / 100`。

#### Scenario: 中立城池比例
- **WHEN** 总城池 20，`neutral_city_ratio = [30, 50]`
- **THEN** 中立城池数为 `20 * 30/100` 到 `20 * 50/100`，即 6-10

### Requirement: 主菜单地图选择
主菜单 SHALL 提供 4 个地图大小按钮（小/中/大/巨大）。点击后设 `NeedsGameReset::NewGame(MapSize)` 并进入游戏。

#### Scenario: 选择 Large 地图
- **WHEN** 用户点击"大"
- **THEN** `NeedsGameReset` 设为 `NewGame(MapSize::Large)`，`GameState` 切换到 `Playing`

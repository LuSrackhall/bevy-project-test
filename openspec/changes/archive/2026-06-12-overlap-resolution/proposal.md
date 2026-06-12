## Why

分离力（separation force）和阵型偏移（formation offset）都是在特定场景下打补丁——前者只在移动中生效、到达后失效，后者只对 waypoint 有效、对战斗无效。需要一道与游戏逻辑完全解耦的底层物理防线：无论重叠如何产生，Tick 结束时强制散开。

## What Changes

- **新增** `overlap_resolution_system`：Tick 末尾的后处理碰撞解算，扫描全部士兵 → 推开重叠 → 迭代 3-5 次至稳定
- **移除** `soldier_movement_system` 中的分离力逻辑（从两遍合并回单遍）
- **移除** `personal_offset` 函数和 formation offset 逻辑
- **保留** `SpatialHash` 工具模块（供 overlap_resolution 复用）
- **保留** 产兵抖动（减少解算器负担）
- **配置**：`unit_separation` → `overlap_resolution`（保留 radius，移除 weight，新增 max_iterations）

## Capabilities

### New Capabilities

- `overlap-resolution`: 后处理重叠解算系统，Tick 末尾保证全体士兵无重叠

### Modified Capabilities

- `unit-separation`: 分离力被重叠解算替代
- `waypoint-formation-offset`: 阵型偏移被重叠解算替代

## Impact

- `crates/simulation/src/soldier/mod.rs`: 新增 `overlap_resolution_system`，移除分离力 + personal_offset
- `crates/simulation/src/soldier/spatial_hash.rs`: 保留不变
- `crates/simulation/src/lib.rs`: tick 管线新增 Phase
- `crates/simulation/src/combat/config.rs`: 配置重构
- `content/combat.ron`: 配置重构

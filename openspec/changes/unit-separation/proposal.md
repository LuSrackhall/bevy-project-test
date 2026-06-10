## Why

士兵移动系统当前无视其他士兵位置，多个士兵可以完全重叠在同一坐标。视觉效果上 N 个兵看起来像 1 个兵，且不符合物理逻辑。RTS 游戏需要单位间的空间分离（separation）来保证可读性和操作性。

## What Changes

- `soldier_movement_system` 新增分离力（separation force）机制：移动后检查周边士兵，若距离过近则推开
- 新增 `SpatialHash` 空间哈希工具：O(n) 构建 + O(1) 邻居查询，支持万人同屏
- `CombatGlobalConfig` 新增 `UnitSeparationConfig { separation_radius, separation_weight }`
- `content/combat.ron` 新增配置段

## Capabilities

### New Capabilities

- `unit-separation`: 士兵单位间的空间分离，防止视觉重叠

### Modified Capabilities

- `simulation-crate`: 士兵移动系统新增分离力计算步骤

## Impact

- `crates/simulation/src/soldier/mod.rs`: `soldier_movement_system` 新增分离力逻辑
- `crates/simulation/src/combat/config.rs`: `CombatGlobalConfig` 新增 `unit_separation` 配置
- `crates/simulation/src/soldier/spatial_hash.rs`: **新文件**，空间哈希工具模块
- `content/combat.ron`: 新增 `unit_separation` 配置段

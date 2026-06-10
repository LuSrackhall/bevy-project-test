## Why

unit-separation 的分离力未能实际生效。两个 Bug：

1. **SpatialHash 存旧位置**：查询邻居时用的是移动前的位置，同位置同方向的兵被推向同一方向，仍然重叠
2. **产兵点完全重叠**：`city_spawn_system` 所有士兵从同一坐标生成，从源头注定堆叠

## What Changes

- `soldier_movement_system`: 改为两遍分离——第一遍收集所有 raw_pos，第二遍用 raw_pos 建 SpatialHash 做分离查询
- `city_spawn_system`: 产兵位置添加确定性小偏移（±8 单位），从源头分散
- `content/combat.ron`: `separation_weight` 从 0.40 调至 0.60

## Capabilities

### Modified Capabilities

- `unit-separation`: 分离力从单遍改为两遍，产兵新增初始分散

## Impact

- `crates/simulation/src/soldier/mod.rs`: `soldier_movement_system` 重构为两遍 + `city_spawn_system` 产兵偏移
- `content/combat.ron`: `separation_weight` 调整

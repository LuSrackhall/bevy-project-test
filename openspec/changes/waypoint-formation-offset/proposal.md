## Why

分离力无法解决多士兵汇聚到同一 waypoint 时的堆叠问题。根本原因是所有兵共享同一个目标坐标——这是一个 n-body 吸引问题，分离力（排斥）永远打不过共同目标（引力），且到达阈值内后分离力不再作用。正确方案是给每个兵分配不同的目标位置（formation offset）。

## What Changes

- `soldier_movement_system`: 对 waypoint 和城池目标应用 `personal_offset(unit_id)`，每个兵获得唯一的目标偏移
- 对敌方兵种目标保留分离力机制（不应用偏移，近战自然成环）
- 新增 `personal_offset()` 函数：32×32 网格，8 单位间距，纯整数确定性计算
- `content/combat.ron`: 可选新增 `formation_spacing: 8` 配置

## Capabilities

### New Capabilities

- `waypoint-formation-offset`: 多单位 waypoint 移动时自动形成方阵，消除重叠

### Modified Capabilities

- `simulation-crate`: `soldier_movement_system` 新增目标偏移逻辑

## Impact

- `crates/simulation/src/soldier/mod.rs`: 新增 `personal_offset()` 函数 + `soldier_movement_system` 偏移逻辑
- `crates/simulation/src/combat/config.rs`: 可选新增 formation_spacing 配置字段
- `content/combat.ron`: 可选新增配置值

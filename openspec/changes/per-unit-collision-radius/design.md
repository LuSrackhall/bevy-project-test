## Context

`overlap_resolution_system` 使用 `OverlapResolutionConfig.min_separation`（默认 8）作为所有兵种的碰撞判定距离。

## Goals / Non-Goals

**Goals:**
- 每兵种独立碰撞半径，配置于 `content/units.ron`
- 解算公式改为 `radius_A + radius_B`

**Non-Goals:**
- 不改变解算算法结构

## Decisions

### D1: 碰撞半径存于 SoldierUnitConfig

```rust
#[serde(default = "default_collision_radius")]
pub collision_radius: u32,
fn default_collision_radius() -> u32 { 6 }
```

serde default 保证旧配置兼容。

### D2: 推荐值

| 兵种 | radius | 理由 |
|------|--------|------|
| Militia | 6 | 基础步兵 |
| Infantry | 7 | 重步兵，稍大 |
| Archer | 5 | 轻装，最小 |
| Cavalry | 10 | 骑兵+马，最大 |

### D3: 解算公式

```
min_dist = radius_A + radius_B
if dist < min_dist:
    overlap = min_dist - dist
    push_A += dir(A→B) * overlap / 2
```

### D4: 移除全局 min_separation

`OverlapResolutionConfig` 只保留 `max_iterations`，`min_separation` 由兵种配置替代。

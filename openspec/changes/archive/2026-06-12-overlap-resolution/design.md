## Context

当前 soldier_movement_system 承载了分离力（separation force）和 formation offset 两套机制，且 spatial_hash 被用于实时查询邻居。后处理方案将这些全部收敛到一个独立系统。

## Goals / Non-Goals

**Goals:**
- 单一防线：一个系统、一个算法，保证 Tick 结束后无重叠
- soldier_movement_system 回归纯粹移动逻辑（单遍）
- 对所有重叠来源（移动、产兵、任何未来机制）生效

**Non-Goals:**
- 不保留分离力和 formation offset

## Decisions

### D1: 迭代式推开

```
for iter in 0..max_iterations (3):
    for each soldier A:
        for each neighbor B within min_sep:
            overlap = min_sep - dist
            dir = (A.pos - B.pos) / dist
            A.pos += dir * overlap / 2
            B.pos -= dir * overlap / 2
    if no displacement → early exit
```

每次迭代各推 50%，多次迭代后平滑散开，不会瞬移。迭代上限防止极端场景（大量兵挤在极小区域）陷入长循环。

### D2: Tick 管线位置

```
Phase 3:   soldier_movement  (纯粹移动，单遍)
Phase 4:   city_spawn        (产兵)
Phase N:   overlap_resolution ← 最后屏障
Phase N+1: melee_attack, archer_attack, arrow_movement, ...
```

放在产兵之后、战斗之前。战斗系统基于正确（无重叠）的位置做距离判定。

### D3: 配置

```ron
overlap_resolution: (
    min_separation: 8,
    max_iterations: 3,
)
```

### D4: 代码清理

- soldier_movement: 移除 Pass1/Pass2 两遍结构，恢复为单遍循环
- 移除 personal_offset 函数 + formation offset 逻辑 + city_ids 集合
- SpatialHash 模块保留不变，供 overlap_resolution 使用

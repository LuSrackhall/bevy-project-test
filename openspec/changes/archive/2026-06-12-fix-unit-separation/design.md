## Context

`unit-separation` 首次实现存在两个结构性缺陷：

1. SpatialHash 在移动前构建，存储的是当前 Tick 开始时的位置。分离查询用 raw_pos（目的地）作为中心，但邻居位置仍是旧的。同位置同方向的士兵无法被推开。
2. 城池产兵无偏移，所有士兵从同一坐标出发。

## Goals / Non-Goals

**Goals:**
- 两遍分离：raw_pos 建网格，每个兵查询其他兵的 raw_pos 做分离
- 产兵确定性抖动：每兵 ±8 单位随机偏移

**Non-Goals:**
- 不改变分离公式本身

## Decisions

### D1: 两遍分离

Pass 1: 收集所有士兵的 raw_pos（移动+速度+目标计算，不存 SpatialHash）
Pass 2: 用 raw_pos 建 SpatialHash，每个兵查询邻居的 raw_pos，计算分离力，得到 final_pos

**理由**: raw_pos 是"我这一 Tick 想去的位置"。两个兵对比 raw_pos 能在同一时间维度上发现冲突。

### D2: 产兵抖动

```rust
let offset_x = (unit_id.0 % 16) as i32 - 8;
let offset_y = ((unit_id.0 >> 4) % 16) as i32 - 8;
let spawn_pos = FixedVec2::new(pos.x + Fixed::from_int(30 + offset_x),
                                 pos.y + Fixed::from_int(offset_y));
```

基于 UnitId 后 8 位做 16×16 网格偏移，完全确定性。

### D3: separation_weight: 0.40 → 0.60

配合两遍分离提升推力强度。

## Context

`overlap_resolution_system` 在 1000 密集单位下耗时 28.2ms。每迭代对每个单位的每个邻居调用 `integer_sqrt`（Newton 法 ~25 次迭代），3 次迭代 × 1000 单位 × 40 邻居 = 120,000 次调用。

## Goals / Non-Goals

**Goals:**
- 28ms → 3-5ms（Phase 1 即可达成）
- 为 5k-10k 单位做准备（Phase 2+3）

**Non-Goals:**
- 改变碰撞规则
- 替换 SpatialHash
- 多线程

## Decisions

### D1: length_squared 早期筛除

```rust
let min_dist_raw = (my_radius + entry.radius) as i64;
let min_dist_sq = min_dist_raw * min_dist_raw * FIXED_ONE;
if dist_sq.0 >= min_dist_sq { continue; }
// 只有真正重叠的对才调用 integer_sqrt
```

数值验证：`dist_sq.0 = 真实距离² × 256`，`min_dist_sq = radius_sum² × 256`，同空间。

### D2: SpatialHash 增量更新

迭代间记录 displacement 的 (entity, old_pos, new_pos)，从旧 cell 移除 + 插入新 cell。需要 SpatialHash 添加 `remove(pos, unit_id)` 方法。

### D3: 自适应迭代退出

```rust
if overlap_count * 100 < total_count { break; } // 纯整数，禁止浮点
```

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| 数值精度错误 | 4 轮评审确认 `* FIXED_ONE` 正确 |
| Phase 3 破坏确定性 | 纯整数比较 |
| 极端集群时 early-out 无效 | 此时收敛快，迭代少 |

# perf-overlap-optimization: overlap_resolution_system 性能优化

## Context

`overlap_resolution_system` 是之前最大的性能瓶颈。Benchmark 数据：

| 场景 | 优化前 | 优化后 |
|------|--------|--------|
| tick/1000_idle | 2.66 ms | 1.30 ms |
| tick/1000_combat | 35.4 ms | **10.6 ms** |
| overlap_resolution/1000 | 28.2 ms | **2.5 ms** |

**实施结果**：Simulation tick 3.4x 加速（35.4ms → 10.6ms）。但用户反馈 1500 单位仍卡顿——瓶颈已从 simulation 转移到渲染层（draw_debug_shapes_system + unit_info_bar_system 每帧遍历所有单位）。下一步需优化 render_view。

## Goals / Non-Goals

**Goals:**
- Phase 1：用 `length_squared` 早期筛除 90-95% 的 non-overlapping 邻居对，避免不必要的 `integer_sqrt`
- Phase 2：SpatialHash 迭代间增量更新（只重新插入 displacement 的单位）
- Phase 3：自适应迭代退出（overlap_count 低于阈值时提前 break）

**Non-Goals:**
- 改变碰撞规则（所有单位互相碰撞，不分阵营）
- 替换 SpatialHash 数据结构
- 多线程并行

## Decisions

### D1: Phase 1 — squared distance early-out

**决策**：在 `integer_sqrt` 之前添加 `length_squared` 粗判。

```rust
let min_dist_raw = (my_radius + entry.radius) as i64;
let min_dist_sq = min_dist_raw * min_dist_raw * FIXED_ONE;
if dist_sq.0 >= min_dist_sq { continue; }
let dist = Fixed(integer_sqrt(dist_sq.0 * FIXED_ONE));
```

**数值精度验证**（经 4 轮评审确认）：
- `length_squared()` 使用 Fixed 乘法 `(a*b)>>8`，所以 `dist_sq.0 = 真实距离² × FIXED_ONE`
- `min_dist_sq = radius_sum² × FIXED_ONE` — 同空间
- 比较简化为 `真实距离² >= radius_sum²` ✅

**预期效果**：120k sqrt → 6-12k sqrt，28ms → ~3-5ms

### D2: Phase 2 — SpatialHash 增量更新

**决策**：迭代间只更新 displacement 的单位，不全量重建。

实现：
- 记录 displacement 的 (entity, old_cell_key, new_pos)
- 从旧 cell 移除，插入新 cell
- 无 displacement 时跳过重建

**确定性保证**：BTreeMap cell 遍历顺序确定，UnitId 排序的 Vec 插入顺序确定。

**预期效果**：节省 2 次全量重建（~2000 inserts）

### D3: Phase 3 — 自适应迭代退出

**决策**：跟踪每迭代的 overlap_count，纯整数比较退出。

```rust
if overlap_count * 100 < total_count { break; }
```

**约束**：必须用纯整数，禁止浮点（确定性）。

**预期效果**：收敛快时 1 次迭代替代 3 次

## Risks / Trade-offs

| 风险 | 缓解 |
|------|------|
| Phase 1 数值精度错误 | 4 轮评审确认 `* FIXED_ONE` 正确 |
| Phase 3 阈值判断破坏确定性 | 纯整数 `overlap_count * 100 < total_count` |
| Phase 2 增量更新遗漏某些 displacement | 保留全量重建作为 fallback（首次迭代） |
| 极端集群（所有单位重叠）时 early-out 无效 | 此时 sqrt 调用不可避免，但 overlap_resolution 本身收敛快 |

## Scaling Thresholds

| 阈值 | 预期 overlap 耗时 | 状态 |
|------|-----------------|------|
| 1000 | 3-5 ms（当前 28ms） | Phase 1 目标 |
| 5000 | 15-25 ms | Phase 1+3 |
| 10000 | 30-50 ms | 需要更激进优化 |

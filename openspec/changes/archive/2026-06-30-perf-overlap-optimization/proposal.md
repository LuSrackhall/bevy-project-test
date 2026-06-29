## Why

`overlap_resolution_system` 是当前最大性能瓶颈：1000 单位密集排列时占 28.2ms（combat tick 的 80%）。根因是 120,000 次 `integer_sqrt` 调用（每对邻居都算，即使不重叠）。Benchmark 验证：tick/1000_combat = 35.4ms，其中 overlap = 28.2ms。

## What Changes

- 在 `integer_sqrt` 前添加 `length_squared` 早期筛除（跳过 90-95% non-overlapping 对）
- SpatialHash 迭代间增量更新（只重新插入 displacement 单位，不全量重建）
- 自适应迭代退出（overlap_count 低于阈值时提前 break）

## Capabilities

### New Capabilities
- `overlap-squared-earlyout`: Phase 1 — length_squared 早期筛除 integer_sqrt
- `overlap-incremental-hash`: Phase 2 — SpatialHash 迭代间增量更新
- `overlap-adaptive-exit`: Phase 3 — 自适应迭代退出

### Modified Capabilities
- `simulation-crate`: soldier/mod.rs overlap_resolution_system 修改

## Impact

- `crates/simulation/src/soldier/mod.rs` — overlap_resolution_system 重构
- `crates/simulation/src/soldier/spatial_hash.rs` — 可能需要 remove 方法支持增量更新
- 107 现有测试 + golden_test 必须通过（确定性不变）

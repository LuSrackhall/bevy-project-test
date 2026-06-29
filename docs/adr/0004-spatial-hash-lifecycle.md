# ADR-004: SpatialHash 生命周期 — 每 Tick 构建 vs 持久化

## 状态

Accepted

## 决策

SpatialHash 保持每 tick 构建（或每 phase 构建），不作为持久化 Resource 跨 tick 维护。

## 理由

1. **简单性**：每 tick 构建保证 SpatialHash 始终反映当前 World 状态，无需处理增量更新的边界情况（entity despawn、position 变更）。
2. **确定性**：从 World 直接构建保证两次运行产生完全相同的 SpatialHash 内容。
3. **当前规模足够**：在 1000-5000 单位规模下，O(N) 构建成本远小于 O(N²) 的全表扫描。优化重点应先放在消除冗余构建（overlap_resolution 复用）和 O(S²) 算法缺陷上。

## 放弃的方案

1. **持久化 SpatialIndex + dirty-flag**：在 entity 移动时增量更新。优点：O(moved) 替代 O(N)。缺点：需要在每个修改 LogicalPosition 的系统后维护 dirty 状态，增加复杂度。当 N > 10,000 且 profiling 证明 SpatialHash 构建是瓶颈时再考虑。
2. **Flat Grid 替代 BTreeMap**：O(1) insert 替代 O(log C)。缺点：需要预分配固定大小数组，浪费内存；cell 粒度固定不如 BTreeMap 灵活。

## 代价

每 tick 的 O(N) 构建成本。在 100k 单位时可能成为瓶颈。

## 修改条件

当 profiling（tracy）显示 SpatialHash 构建消耗 > 30% tick 预算时，迁移到持久化 SpatialIndex。

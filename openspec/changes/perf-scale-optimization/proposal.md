## Why

当前项目在 1000 单位规模下出现持续帧率下降，目标是支撑 10 万 ~ 100 万单位。经过代码审查发现三个结构性性能缺陷：combat_engagement_system 的 O(S²) 线性扫描、overlap_resolution_system 每 tick 3 次 SpatialHash 重建、每 tick 12+ 次冗余 HashMap 全表构建。同时缺少 profiling 基础设施，无法量化优化效果和防止性能回归。

## What Changes

- 修复 `combat_engagement_system` 的 `Vec::find` O(S²) 为 HashMap O(1) 查找
- 提取 `build_soldier_index` 辅助函数消除 12+ 次重复 HashMap 构建代码
- `overlap_resolution_system` 的 SpatialHash 在迭代间复用（3 次构建 → 1 次）
- 新增 `SpatialHash::query_range(pos, radius)` 泛化接口，替代硬编码 3×3 邻域
- `UnitIdEntityIndex` 改为 spawn/despawn 增量更新，不再每 tick 全量重建
- bevy_adapter 添加 tracing 插桩 + tracy subscriber（feature-gated）
- render_view 的 debug shapes 和 info bars 改为 `debug_render` feature gate
- 新建 `crates/bench/` 独立 benchmark binary（criterion + CI 回归门）
- 补齐宪法 §4.3 复杂度声明、docs/performance.md、ADR-004/005

## Capabilities

### New Capabilities
- `perf-structural-fixes`: Phase 1 结构性 bug 修复（O(S²) engagement、HashMap 去重、overlap SpatialHash 复用）
- `spatial-hash-query-range`: Phase 2 SpatialHash query_range 泛化接口
- `unit-index-incremental`: Phase 3 UnitIdEntityIndex 增量化
- `profiling-infrastructure`: Phase 4 bevy_adapter tracing 插桩 + render_view debug_render feature gate
- `benchmark-crate`: Phase 5 crates/bench/ 独立 binary + CI 回归
- `perf-compliance`: Phase 6 宪法合规补齐（§4.3 + docs/performance.md + ADRs）

### Modified Capabilities
- `simulation-crate`: Phase 1-3 修改 simulation 内部系统和数据结构
- `combat-fixes`: Phase 1a 修改 combat_engagement_system
- `bevy-adapter-crate`: Phase 4 添加 tracing 依赖和插桩
- `render-view-crate`: Phase 4 添加 debug_render feature gate

## Impact

- **Simulation crate**：Phase 1-3 涉及 combat/mod.rs、soldier/mod.rs、soldier/spatial_hash.rs、unit_index.rs 的重构
- **bevy_adapter crate**：Phase 4 添加 tracing + tracing-tracy 依赖
- **render_view crate**：Phase 4 添加 debug_render feature 到 Cargo.toml 和系统注册
- **新增 crate**：Phase 5 创建 crates/bench/ 独立 binary
- **确定性**：保持 BTreeMap + sorted Vec，禁止 HashMap 替代 SpatialHash（已验证 breakage scenario）
- **层拓扑**：UnitIdMapper（bevy_adapter）和 UnitIdEntityIndex（simulation）保持分离

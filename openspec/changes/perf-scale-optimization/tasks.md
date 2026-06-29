## 1. 结构性 Bug 修复 — engagement O(S²) + HashMap 去重

- [x] 1.1 将 combat_engagement_system 中 soldiers Vec 替换为 HashMap<UnitId, SoldierData>，消除 Vec::find O(S²) 线性扫描
- [x] 1.2 提取 `build_soldier_index(world) -> HashMap<UnitId, SoldierSnapshot>` 辅助函数，定义 SoldierSnapshot 结构体
- [x] 1.3 将 melee_attack_system、archer_attack_system、arrow_movement_system、combat_engagement_system 中的独立 HashMap 构建替换为 build_soldier_index 调用
- [x] 1.4 验证所有系统仍独立调用 build_soldier_index（不共享 Resource）

## 2. 结构性 Bug 修复 — overlap_resolution SpatialHash 复用

- [x] 2.1 将 overlap_resolution_system 的 SpatialHash 构建移至迭代循环外，首次构建后复用
- [x] 2.2 添加位置变化检测：仅在迭代确实移动了单位时才重建 SpatialHash
- [x] 2.3 运行现有测试验证 overlap_resolution 行为不变

## 3. SpatialHash query_range 接口

- [x] 3.1 在 SpatialHash 中实现 `query_range(pos: FixedVec2, radius: i64)` 方法，泛化 cell sweep
- [x] 3.2 保留 query_nearby 作为向后兼容接口
- [x] 3.3 为 archer_attack_system 添加使用 query_range 的路径（cell_size=200 时仍用 query_nearby 足够，但接口统一）
- [x] 3.4 添加 query_range 的单元测试：小半径（9 cells）和大半径（49 cells）

## 4. UnitIdEntityIndex 增量化

- [x] 4.1 移除 run_tick 中的 UnitIdEntityIndex::rebuild 全量重建
- [x] 4.2 在 consume_commands_system 的 spawn 处理中添加 index.insert
- [x] 4.3 在所有 despawn 路径（melee_attack、attack_windup、city_interaction、arrow_movement）中添加 index.remove
- [x] 4.4 保留 find_entity_by_unit_id 中的 world.get_entity().is_ok() 安全网
- [x] 4.5 运行 golden_test 和 scenario_test 验证确定性不变

## 5. Profiling 基础设施

- [x] 5.1 在 bevy_adapter/Cargo.toml 添加 tracing 和 tracing-tracy 依赖（feature-gated）
- [x] 5.2 在 simulation_driver_system 的 run_tick_default() 调用周围添加 tracing::info_span
- [x] 5.3 注册 tracing-tracy subscriber（feature-gated）
- [x] 5.4 在 render_view/Cargo.toml 添加 debug_render feature，gate draw_debug_shapes_system 和 unit_info_bar_system

## 6. Benchmark Crate + CI

- [x] 6.1 创建 crates/bench/ 独立 binary crate，添加 criterion 依赖
- [x] 6.2 实现完整 tick benchmark（empty world、1k idle、1k combat、10k idle）
- [x] 6.3 实现 per-phase benchmarks（combat_engagement、melee_attack、soldier_movement、overlap_resolution）
- [x] 6.4 在 CI 中添加 cargo bench + baseline 比较步骤（5% 回归 = 失败）

## 7. 合规补齐

- [ ] 7.1 为所有 hot system 添加 §4.3 Complexity/Memory/Hot-Path doc-comments
- [ ] 7.2 创建 docs/performance.md（基线数据、优化历史、scaling 阈值）
- [ ] 7.3 创建 docs/adr/0004-spatial-hash-lifecycle.md（持久化 vs 每 tick 重建）
- [ ] 7.4 创建 docs/adr/0005-phase-dependency-graph.md（阶段依赖与并行策略）

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run myspec-verify skill to verify implementation
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, run myspec-merge skill
4. **Archive**: Handled by myspec-merge
5. **Cleanup**: Handled by myspec-merge

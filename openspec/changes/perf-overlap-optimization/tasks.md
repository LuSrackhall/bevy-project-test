## 1. Phase 1 — length_squared 早期筛除

- [ ] 1.1 在 overlap_resolution_system 碰撞检测循环中，`integer_sqrt` 前添加 `min_dist_sq` 早期筛除
- [ ] 1.2 运行 golden_test 验证确定性不变
- [ ] 1.3 运行 `cargo bench -p bench --bench phase_bench` 验证 overlap 耗时下降

## 2. Phase 2 — SpatialHash 增量更新

- [ ] 2.1 在 SpatialHash 中添加 `remove(pos, unit_id)` 方法
- [ ] 2.2 重写 overlap_resolution_system 迭代间逻辑：记录 displacement，增量更新 SpatialHash
- [ ] 2.3 运行测试验证碰撞行为不变

## 3. Phase 3 — 自适应迭代退出

- [ ] 3.1 在迭代循环中跟踪 overlap_count
- [ ] 3.2 添加纯整数退出条件 `overlap_count * 100 < total_count`
- [ ] 3.3 运行完整测试套件

---

## Post-Implementation Workflow

After completing ALL tasks above:

1. **Verify**: Run myspec-verify skill
2. **User Acceptance**: Present results, ask user to confirm
3. **Merge**: After user accepts, run myspec-merge skill

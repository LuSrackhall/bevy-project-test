## 1. TickCombatIndex Resource

- [x] 1.1 定义 TickCombatIndex 结构体（soldiers, soldier_spatial, faction_indices）
- [x] 1.2 在 run_tick 开始时构建 TickCombatIndex，插入为 Resource
- [x] 1.3 修改 combat_engagement_system 读取共享索引
- [x] 1.4 修改 melee_attack_system 读取共享索引
- [x] 1.5 修改 archer_attack_system 读取共享索引
- [x] 1.6 修改 arrow_movement_system 读取共享索引
- [x] 1.7 运行 golden_test 验证确定性不变

## 2. 按阵营 SpatialHash

- [ ] 2.1 在 TickCombatIndex 构建时按 Faction 分别构建 SpatialHash
- [ ] 2.2 combat 系统查询时只扫描敌方阵营 SpatialHash
- [ ] 2.3 运行 benchmark 验证性能改善

---
## Post-Implementation Workflow

1. **Verify**: Run myspec-verify
2. **User Acceptance**: Present results
3. **Merge**: After user accepts, run myspec-merge

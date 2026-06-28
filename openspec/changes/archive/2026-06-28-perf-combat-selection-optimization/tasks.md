## 1. SpatialHash 改造

- [x] 1.1 将 `spatial_hash.rs` 中 HashMap 改为 BTreeMap
- [x] 1.2 SpatialHash 的 cell 数据扩展为包含 UnitId（当前只存 FixedVec2 + u32）
- [x] 1.3 构建时每个 cell 内 Vec 按 UnitId 排序
- [x] 1.4 更新 insert 和 query_nearby 签名
- [x] 1.5 更新 overlap_resolution_system 调用适配新签名
- [x] 1.6 运行 `cargo test -p simulation` 确认现有测试通过

## 2. UnitIdEntityIndex

- [x] 2.1 定义 `UnitIdEntityIndex(HashMap<UnitId, Entity>)` Resource
- [x] 2.2 在 run_tick Step 5 开头重建索引（全量扫描 UnitIdComponent）
- [x] 2.3 替换 soldier/mod.rs 中 find_entity_by_unit_id 调用为 index 查找
- [x] 2.4 替换 combat/mod.rs 中 find_entity_by_unit_id 调用为 index 查找
- [x] 2.5 运行 `cargo test -p simulation` 确认测试通过

## 3. combat_engagement_system 优化

- [x] 3.1 sorted_ids 排序从每士兵循环内移到循环外（一次排序）
- [x] 3.2 运行 `cargo test -p simulation` 确认测试通过

## 4. Bug 修复

- [x] 4.1 修复 city_interaction_system：UnitDestroyed 事件中 unit_id 从 UnitId(0) 改为实际值
- [x] 4.2 修复 arrow_movement_system：箭矢衰减销毁时补发 UnitDestroyed 事件
- [x] 4.3 运行 `cargo test -p simulation` 确认测试通过

## 5. render_view 优化

- [x] 5.1 selection_visual_system 用 find_entity_by_unit_id O(1) 替代 O(m*n) 扫描
- [x] 5.2 HUD 中 find_entity_by_unit_id 调用改用 simulation 的 O(1) 实现
- [x] 5.3 selection_visual_system 处理已 despawn 的 Entity（get_entity Result）
- [x] 5.4 find_entity_by_unit_id 验证 Entity 存活状态后再返回
- [x] 5.5 运行 `cargo test` 全项目确认测试通过

## 6. 最终验证

- [x] 6.1 运行 `cargo test` 全项目确认无编译错误和测试失败
- [x] 6.2 确认 SpatialHash 使用 BTreeMap + cell 内排序

## 后续变更（不在本次范围）

- [ ] 4 个战斗系统推广 SpatialHash（combat_engagement、melee、archer、arrow）— 需预构建 faction_map 解决借用冲突

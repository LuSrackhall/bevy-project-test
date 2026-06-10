## 1. 配置层

- [x] 1.1 `CombatGlobalConfig` 新增 `UnitSeparationConfig { separation_radius: u32, separation_weight: f32 }`
- [x] 1.2 `content/combat.ron` 新增 `unit_separation` 配置段（radius=8, weight=0.40）

## 2. 空间哈希

- [x] 2.1 新建 `crates/simulation/src/soldier/spatial_hash.rs`：`SpatialHash` 结构体（insert / query_nearby）
- [x] 2.2 导入到 `soldier/mod.rs`

## 3. 分离力

- [x] 3.1 `soldier_movement_system` 新增分离力逻辑：构建 SpatialHash → 计算目标位置 → 查询邻居 → 调整位置

## 4. 验证

- [x] 4.1 `cargo check` 编译通过
- [x] 4.2 `cargo test` 所有测试通过

## 1. 配置层

- [x] 1.1 `SoldierUnitConfig` 新增 `collision_radius: u32`（serde default = 6）
- [x] 1.2 `content/units.ron` 各兵种添加 `collision_radius`（militia=6, infantry=7, archer=5, cavalry=10）
- [x] 1.3 `OverlapResolutionConfig` 移除 `min_separation`；`content/combat.ron` 同步移除

## 2. 解算逻辑

- [x] 2.1 `overlap_resolution_system`: 查询 `SoldierTypeComponent` + `SoldierConfig`，使用 `radius_A + radius_B` 替代全局 `min_separation`

## 3. 验证

- [x] 3.1 `cargo check` 编译通过
- [x] 3.2 `cargo test` 所有测试通过

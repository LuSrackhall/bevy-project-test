## 1. 配置重构

- [x] 1.1 `CombatGlobalConfig`: `unit_separation` → `overlap_resolution { min_separation: u32, max_iterations: u32 }`
- [x] 1.2 `content/combat.ron`: 更新配置段

## 2. 新增 overlap_resolution_system

- [x] 2.1 `soldier/mod.rs`: 实现 `overlap_resolution_system`（迭代推开算法 + SpatialHash）
- [x] 2.2 `lib.rs`: tick 管线 `city_spawn` 之后插入 `overlap_resolution`

## 3. 代码清理

- [x] 3.1 `soldier_movement_system`: 移除分离力逻辑（恢复单遍），移除 personal_offset + formation offset
- [x] 3.2 移除 `personal_offset` 函数

## 4. 验证

- [x] 4.1 `cargo check` 编译通过
- [x] 4.2 `cargo test` 所有测试通过

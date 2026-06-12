## 1. 偏移函数

- [x] 1.1 `soldier/mod.rs` 新增 `personal_offset(unit_id, spacing) -> (i32, i32)` 函数

## 2. 移动系统改造

- [x] 2.1 `soldier_movement_system`: waypoint 和城池目标在 `target_pos` 计算时叠加 personal_offset
- [x] 2.2 `soldier_movement_system`: 确保敌方兵种目标不应用偏移（保留分离力路径）

## 3. 验证

- [x] 3.1 `cargo check` 编译通过
- [x] 3.2 `cargo test` 所有测试通过

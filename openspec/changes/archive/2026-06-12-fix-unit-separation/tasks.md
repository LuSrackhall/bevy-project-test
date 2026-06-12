## 1. 两遍分离

- [x] 1.1 `soldier_movement_system`: 重构为两遍 — Pass 1 收集 raw_pos 到 Vec，Pass 2 用 raw_pos 建 SpatialHash + 分离力 + 写入 final_pos

## 2. 产兵抖动

- [x] 2.1 `city_spawn_system`: spawn_pos 基于 UnitId 添加确定性偏移（±8 范围）

## 3. 配置微调

- [x] 3.1 `content/combat.ron`: `separation_weight` 从 `0.40` 调至 `0.60`

## 4. 验证

- [x] 4.1 `cargo check` 编译通过
- [x] 4.2 `cargo test` 所有测试通过

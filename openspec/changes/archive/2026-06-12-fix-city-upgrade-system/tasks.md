## 1. 配置层

- [x] 1.1 `content/cities.ron`: `level_up_cost_multiplier` 从 `100.0` 改为 `1.0`

## 2. 升级逻辑修复

- [x] 2.1 `city_interaction_system` 升级分支：`req` 公式改为 `max_hp × cost_multiplier × level`
- [x] 2.2 `city_interaction_system` 升级分支：新增 `max_population = level × base_population_per_level` 更新
- [x] 2.3 `city_interaction_system` 升级分支：新增 `CityRadius` 更新（`visual_radius_base + level × visual_radius_per_level`）
- [x] 2.4 `update_bottom_panel` exp_text 适配新公式（如需要）

## 3. 验证

- [x] 3.1 `cargo check` 编译通过
- [x] 3.2 `cargo test` 所有测试通过

## 1. 修复经验显示

- [ ] 1.1 `update_bottom_panel` 新增 `exp_text` 更新：读取 `sim_world` 中的 `CityGlobalConfig`，计算 `required_exp = city.health_max × level_up_cost_multiplier`，写入 `"经验 {city.level_exp}/{required_exp}"`
- [ ] 1.2 `cargo check` 编译通过
- [ ] 1.3 `cargo test` 现有测试通过

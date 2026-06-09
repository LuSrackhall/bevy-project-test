## 1. 组件与配置层

- [ ] 1.1 `CityComponent` 新增 `arrow_damage_acc: u32` 字段，所有创建/易手点初始化为 0
- [ ] 1.2 `CombatGlobalConfig` 新增 `arrow_building_damage_ratio: f32` 字段
- [ ] 1.3 `content/combat.ron` 新增 `arrow_building_damage_ratio: 0.005` 配置值

## 2. 城池碰撞检测实现

- [ ] 2.1 `arrow_movement_system` 新增城池实体收集查询（`CityMarker` + `LogicalPosition` + `FactionComponent` + `CityRadius`），紧接在兵种查询之后
- [ ] 2.2 在兵种碰撞循环之后新增城池碰撞循环：检查 `city_faction != arrow.from_faction` 且在 `CityRadius` 内，命中后执行城池伤害逻辑并强制 decay
- [ ] 2.3 确保城池碰撞仅在箭矢未被兵种停止时执行（穿透成功或无兵种命中）

## 3. 伤害累积逻辑

- [ ] 3.1 实现城池箭矢伤害累积：`arrow_damage_acc += arrow.damage`，随后 `integer_damage = acc / 200`，若 > 0 则减 `health_current` 并 `acc %= 200`
- [ ] 3.2 城池建筑碰撞后不再进行穿透判定，直接设置 `decay_remaining = ARROW_DECAY_TICKS`

## 4. 验证与收尾

- [ ] 4.1 `cargo check` 编译通过
- [ ] 4.2 `cargo test` 所有现有测试通过
- [ ] 4.3 编写新单元测试：箭矢命中城池 → 累加器递增但不扣血（值不足）
- [ ] 4.4 编写新单元测试：箭矢累积命中城池 → 累加值满 200 后扣血
- [ ] 4.5 编写新单元测试：箭矢命中城池后进入衰减
- [ ] 4.6 编写新单元测试：己方箭矢穿过己方城池
- [ ] 4.7 编写新单元测试：穿透兵种后的箭矢同一 Tick 命中后方城池

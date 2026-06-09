## 1. 配置更新

- [x] 1.1 更新 `content/units.ron` archer 条目：新增 `arrow_speed: 40`、`attack_range_base: 380`、`attack_range_max: 600`、`attack_range_max_level: 4`、`overshoot_base: 20`、`overshoot_per_level: 20`、`pierce_base_chance: 0.05`、`pierce_per_level: 0.02`、`pierce_unlock_level: 2`
- [x] 1.2 更新 `SoldierUnitConfig` 新增对应字段 + serde Deserialize

## 2. Arrow 组件重构

- [x] 2.1 重写 `Arrow` 组件：新增 `direction: FixedVec2`、`flight_remaining: u32`、`decay_remaining: u32`、`pierce_chance: f32`、`stuck_to: Option<UnitId>`、`hit_units: Vec<UnitId>`。删除 `target: UnitId`、`remaining_ticks: u32`
- [x] 2.2 在 `init_simulation_world` 中预计算 `max_flight_ticks`，作为资源存储

## 3. 新增 arrow_movement_system

- [x] 3.1 实现 `arrow_movement_system`：每 Tick 递减 `flight_remaining` 或 `decay_remaining`，更新位置，碰撞检测
- [x] 3.2 碰撞检测：遍历所有敌方单位，距离 < 碰撞半径时造成伤害，加入 hit_units
- [x] 3.3 穿透逻辑：发射时预计算 `pierce_chance`，碰撞后判定——穿透成功继续飞行，失败进入衰减
- [x] 3.4 衰减逻辑：`decay_remaining > 0` 时递减，跟随 stuck_to 目标位置；归零时销毁

## 4. 重写 archer_attack_system

- [x] 4.1 攻击距离改为运行时计算：`attack_range = attack_range_base + (level - 1) × (attack_range_max - attack_range_base) / (attack_range_max_level - 1)`
- [x] 4.2 超射计算：`overshoot = overshoot_base + (level - 1) × overshoot_per_level`
- [x] 4.3 飞行 Tick：`flight_ticks = (attack_range + overshoot) / arrow_speed`
- [x] 4.4 方向向量：`direction = (target_pos - archer_pos).normalized() × arrow_speed`（箭速编码在方向量级中）
- [x] 4.5 穿透几率：`if level >= pierce_unlock_level { pierce_base_chance + (level - pierce_unlock_level) × pierce_per_level } else { 0.0 }`

## 5. 删除旧系统

- [x] 5.1 删除 `arrow_hit_system`
- [x] 5.2 删除 `arrow_expire_system`

## 6. 更新 run_tick 阶段顺序

- [x] 6.1 在 `archer_attack_system` 之后插入 `arrow_movement_system`
- [x] 6.2 删除 `arrow_hit_system` 和 `arrow_expire_system` 的调用

## 7. 渲染更新

- [x] 7.1 `debug_shape.rs`: 箭矢渲染根据 `decay_remaining` 计算 alpha 透明度
- [x] 7.2 衰减阶段箭矢半径从 4px 线性缩小至 1px

## 8. 测试验证

- [x] 8.1 `cargo test -p simulation` 全部通过
- [x] 8.2 `cargo check --workspace` 零错误
- [x] 8.3 手动验证：弓兵发射箭矢可见（飞行 > 0.5 秒），命中后衰减动画可见

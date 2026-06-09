## Context

当前箭矢物理模型：`Arrow { target, remaining_ticks }`——箭矢跟踪目标飞行，5 Tick 后判伤。该模型有两个缺陷：飞行时间过短肉眼不可见、跟踪制导不符合弹道直觉。

## Goals / Non-Goals

**Goals:**
- 箭矢沿固定方向飞行，直至达到最大距离或碰撞敌方单位
- 箭矢生命周期 > 1 秒（包含飞行阶段 + 1 秒衰减动画）
- 支持穿透机制（碰撞后概率继续飞行）
- 弓箭手攻击距离随等级成长（380→600）

**Non-Goals:**
- 不改变其他兵种的战斗逻辑
- 不引入弓箭手 AI 行为变更（已不参与 combat_engagement 自动索敌）
- 不改变城池攻击逻辑

## Decisions

### 决策 1: Arrow 组件模型

```rust
pub struct Arrow {
    pub direction: FixedVec2,     // 发射时的固定方向（单位向量×箭速）
    pub speed: u32,               // 每 Tick 飞行距离
    pub damage: u32,
    pub from_faction: Faction,
    pub shooter: Option<UnitId>,
    pub flight_remaining: u32,    // 飞行剩余 Tick
    pub decay_remaining: u32,     // 衰减剩余 Tick（0=飞行中）
    pub pierce_chance: f32,       // 发射时预计算的穿透率
    pub stuck_to: Option<UnitId>, // 衰减阶段附着目标
    pub hit_units: Vec<UnitId>,   // 已命中单位（防同 Tick 重复命中）
    pub start_pos: FixedVec2,
}
```

### 决策 2: 飞行 Tick 预计算

计算公式：`flight_ticks = (attack_range + overshoot) / arrow_speed`

所有值在 `init_simulation_world` 时锁定，运行时仅递减 `flight_remaining`，无除法运算。

| Lv | 射程 | 超射 | 总飞行距离 | 飞行 Tick（以箭速=40为例） |
|----|------|------|-----------|--------------------------|
| 1 | 380 | 20 | 400 | 10 |
| 2 | 453 | 40 | 493 | 12 |
| 3 | 527 | 60 | 587 | 14 |
| 4 | 600 | 80 | 680 | 17 |

射程公式: `380 + (level - 1) × 220 / 3`（取整）
超射公式: `20 + (level - 1) × 20`

### 决策 3: 穿透几率

- Lv.1: 0%
- Lv.2+: `5% + (level - 2) × 2%`，无上限

发射时掷一次骰子（`gen_probability() < pierce_chance`），结果存入 `pierce_chance` 字段。若穿透成功 flag 为 true，碰撞后继续飞行。

### 决策 4: 每 Tick 阶段逻辑

```
flight_remaining > 0:
  pos += direction × speed
  flight_remaining -= 1
  for each enemy unit within collision_radius:
    if unit not in hit_units:
      deal damage
      hit_units.push(unit.id)
      if pierce_check(level) == SUCCESS:
        continue flying  // 不进入衰减
      else:
        stuck_to = unit
        decay_remaining = DECAY_TICKS (20)
        break flight

flight_remaining == 0 && decay_remaining == 0:
  decay_remaining = DECAY_TICKS (20)  // 自然衰减

decay_remaining > 0:
  decay_remaining -= 1
  if stuck_to: pos = stuck_to.pos  // 跟随
  render with shrinking alpha  // 视觉层处理

decay_remaining == 0:
  despawn
```

### 决策 5: 系统合并

删除 `arrow_hit_system` 和 `arrow_expire_system`，合并为 `arrow_movement_system`：
- 飞行移动 + 碰撞检测
- 衰减倒计时
- 销毁箭矢

### 决策 6: 攻击距离配置

`content/units.ron` 中 archer 条目新增：
```ron
"archer": (
    ...
    arrow_speed: 40,
    attack_range_base: 380,
    attack_range_max: 600,
    attack_range_max_level: 4,
    overshoot_base: 20,
    overshoot_per_level: 20,
    pierce_base_chance: 0.05,
    pierce_per_level: 0.02,
    pierce_unlock_level: 2,
)
```

`SoldierUnitConfig` 新增对应字段，`attack_range` 改为运行时根据等级计算。

### 决策 7: Tick Pipeline 顺序调整

```
相位 2.5: arrow_movement（在 archer_attack 之后立即运行）
```

## Risks / Trade-offs

- **[性能]** 每 Tick 箭矢碰撞检测为 O(arrows × units)，在千人规模下可能成为瓶颈。→ 缓解：使用网格分区（未来优化），当前单位数 < 200 时直接遍历足够
- **[穿透雪球]** 无上限穿透率可能导致高等级弓兵过于强大。→ 缓解：穿透后不重复命中同一单位（hit_units 去重）
- **[配置兼容]** 新增字段需要更新 `SoldierUnitConfig` 和 RON 文件。→ 旧配置不兼容，需要手动更新

## Migration

- 删除 `arrow_hit_system` 和 `arrow_expire_system`
- 新增 `arrow_movement_system`
- 修改 `archer_attack_system` — 方向发射 + 预计算 ticks
- 更新 `run_tick` 阶段顺序
- 更新 `content/units.ron` archer 条目
- 更新 `debug_shape.rs` 支持衰减渲染

# ADR-005: Phase 依赖图与未来并行策略

## 状态

Accepted

## 决策

当前 17 个 simulation phase 保持顺序执行。Phase 之间的数据依赖关系记录如下，为未来可能的并行化提供依据。

## Phase 依赖图

```
Phase 1:  consume_commands        (depends on: CommandBuffer)
Phase 2:  combat_engagement       (depends on: all soldier positions)
Phase 2.5: facing_turn            (depends on: Movement.waypoint)
Phase 3:  soldier_movement        (depends on: Movement, positions)
Phase 4:  city_spawn              (depends on: CityComponent)
Phase 4.5: overlap_resolution     (depends on: positions after movement)
Phase 5:  city_capture_check      (depends on: positions, factions)
Phase 6:  city_interaction        (depends on: positions, CityComponent)
Phase 6.5: shield_pickup          (depends on: positions, DroppedShield)
Phase 7:  aura_heal               (depends on: Health, positions)
Phase 8:  melee_attack            (depends on: positions, Health, Attack)
Phase 8.5: attack_windup          (depends on: AttackWindup, positions)
Phase 9:  archer_attack           (depends on: positions, Attack for archers)
Phase 10: arrow_movement          (depends on: Arrow, soldier positions)
Phase 11: slow_debuff_tick        (depends on: SlowDebuff)
Phase 12: fearless_buff_tick      (depends on: FearlessBuff)
Phase 13: soldier_level_up        (depends on: Level.exp)
Phase 13.5: shield_decay          (depends on: DroppedShield)
Phase 14: ai_decide               (depends on: all state)
```

## 可并行化组（理论分析，当前不实现）

| Group | Phases | 依赖关系 |
|-------|--------|---------|
| Independent buffs | 11, 12, 13 | 互不依赖，只读各自 component |
| Post-combat cleanup | 11, 12, 13, 13.5 | 在 combat 之后，互相独立 |

**注意**：Phase 8-10（combat chain）有严格顺序依赖，不可并行。

## 理由

1. **确定性优先**（§0.1）：顺序执行保证完全确定性。并行执行需要额外的同步机制来保证确定性（§13.5）。
2. **当前规模不需要**：在 1000-5000 单位下，顺序执行的 17 个 O(N) phase 总成本约为 2-5ms，远低于 50ms tick 预算。
3. **Bevy schedule 不适用**：simulation 层直接调用函数，不使用 Bevy 的 schedule 系统。并行化需要在 run_tick 内部实现。

## 代价

在 100k+ 单位时，顺序执行可能成为瓶颈。

## 修改条件

当单 tick 耗时 > 10ms（20% tick 预算）且 profiling 显示 phase 执行（非 SpatialHash 构建）是主要成本时，考虑引入 phase 并行。

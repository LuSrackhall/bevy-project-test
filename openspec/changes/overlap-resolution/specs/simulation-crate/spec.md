## MODIFIED Requirements

### Requirement: 固定 Tick 仿真调度

simulation crate SHALL 在 `run_tick(world, tick_number)` 函数中按固定顺序执行仿真阶段。阶段顺序 SHALL 为：consume_commands → combat_engagement → soldier_movement → city_spawn → overlap_resolution → city_capture_check → city_interaction → aura_heal → melee_attack → archer_attack → arrow_movement → slow_debuff_tick → fearless_buff_tick → soldier_level_up → ai_decide。SHALL NOT 依赖帧率或系统时钟。

#### Scenario: Tick 顺序确定性

- **WHEN** 以相同世界状态和相同命令输入执行 Tick N 两次
- **THEN** 两次执行后的世界状态完全相同（逐组件逐字段一致）

#### Scenario: 无帧率依赖

- **WHEN** 两次 `run_tick` 调用之间有任意时间间隔
- **THEN** 仿真结果仅取决于传入的 `tick_number` 参数和当前世界状态，不受实际间隔时间影响

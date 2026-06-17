## MODIFIED Requirements

### Requirement: Tick 调度驱动
bevy_adapter SHALL 提供 `TickClock` 资源和 `tick_driver` 系统。`tick_driver` SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时运行。每帧累加 `time.delta_secs()`，当 `accumulator >= tick_duration` 时执行一次完整 Tick。

#### Scenario: 固定频率触发
- **WHEN** `tick_duration = 50ms`，而帧时间累积达到 100ms，且 `GameState::Playing` 且未暂停
- **THEN** `tick_driver` 连续执行 2 次完整 Tick，`accumulator` 剩余 < 50ms

#### Scenario: 主菜单时不 tick
- **WHEN** `GameState::MainMenu`
- **THEN** `tick_driver` 不运行，`TickClock` 不变化

#### Scenario: 暂停时不 tick
- **WHEN** `GameState::Playing` 且 `Paused.0 == true`
- **THEN** `tick_driver` 不运行，`TickClock` 不变化

### Requirement: 实体生灭同步
bevy_adapter SHALL 通过 `sync_entities_system` 监听 simulation 层事件，在 Bevy 世界中同步创建/销毁对应实体。`sync_entities_system` SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时运行。`backfill_entities_system` SHALL 由 `reset_game_system` 调用（从 `Startup` 移除），在 `OnEnter(Playing)` 时执行。

#### Scenario: 新实体同步
- **WHEN** `GameState::Playing` 且未暂停，simulation 层产出新实体
- **THEN** `sync_entities_system` 创建对应 Bevy 实体并注册到 `UnitIdMapper`

#### Scenario: 暂停时不同步
- **WHEN** `GameState::Playing` 且 `Paused.0 == true`
- **THEN** `sync_entities_system` 不运行

### Requirement: UnitIdMapper 清理方法
`UnitIdMapper` SHALL 提供 `clear()` 方法，清空 `unit_to_entity` 和 `entity_to_unit` 两个映射表。

#### Scenario: 清空映射
- **WHEN** `reset_game_system` 调用 `mapper.clear()`
- **THEN** `unit_to_entity` 和 `entity_to_unit` 均为空 `HashMap`

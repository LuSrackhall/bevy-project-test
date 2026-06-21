## MODIFIED Requirements

### Requirement: Tick 调度驱动

bevy_adapter SHALL 提供 `TickClock` 资源、`GameActive(bool)` 资源和 `Paused(bool)` 资源。bevy 版本 SHALL 为 `0.19`。`insert_non_send_resource` 调用 SHALL 更新为 `insert_non_send`（Bevy 0.19 API 重命名）。

#### Scenario: 固定频率触发

- **WHEN** `tick_duration = 50ms`，而帧时间累积达到 100ms，且 `GameState::Playing` 且未暂停
- **THEN** `tick_driver` 连续执行 2 次完整 Tick，`accumulator` 剩余 < 50ms

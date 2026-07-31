## Why

移动指令时灵时不灵，需多次点击。根因是命令指向 `current_tick+1`，relay 可能已 finalize 该 tick 而丢弃命令。`input_delay` 从未被使用。

## What Changes

- `SimulationDriver::command_delay()` — Network 返回 input_delay，其他模式返回 1
- 命令目标 tick 改为 `current_tick + command_delay()`
- `network_flush_system` 发送窗口帧 `[current_tick+1, current_tick+input_delay]`
- 输入系统 `.before(network_flush_system)` 排序约束
- 新增双客户端可靠性 e2e 测试

## Capabilities

### New Capabilities
- `command-delivery-reliability`: 网络命令可靠送达

### Modified Capabilities
无。

## Impact

- `crates/bevy_adapter/src/driver.rs`
- `crates/bevy_adapter/src/transport.rs`
- `crates/render_view/src/selection.rs`
- `crates/render_view/src/lib.rs`
- `crates/bevy_adapter/tests/network_move_e2e.rs`（新增）

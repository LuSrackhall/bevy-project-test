## Context

详见 brainstorm-spec.md。命令指向已 finalize 的 tick 被丢弃。

## Goals / Non-Goals

**Goals:**
- 命令指向未来 tick，不被 relay 丢弃
- 保持 relay 顺序推进（窗口帧）

**Non-Goals:**
- 不修改 relay 协议

## Decisions

### command_delay

`SimulationDriver::command_delay()`: Network → input_delay，Live/Replay → 1。

### 窗口发送

`network_flush_system` 对 `[current_tick+1, current_tick+input_delay]` 每个 tick 发一个帧，`take_for_tick(tick)` 取命令。中间 tick 为空帧保持 relay 推进。

### 排序

输入系统组 `.before(bevy_adapter::transport::network_flush_system)`。

## Risks / Trade-offs

- [Risk] 3 tick 命令延迟 → 网络缓冲必然代价

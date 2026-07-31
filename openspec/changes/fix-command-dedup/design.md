## Context

详见 brainstorm-spec.md。`network_flush_system` 命令残留导致重复发送。

## Goals / Non-Goals

**Goals:**
- 每个 tick 的每个 player 只发送一次命令
- relay 不积累重复指令

**Non-Goals:**
- 不修改 relay 协议
- 不修改 tick 推进逻辑

## Decisions

### Client drain

`transport.rs` 中 `network_flush_system`：
- `Res<CommandBuffer>` → `ResMut<CommandBuffer>`
- `iter().filter(|c| c.tick == cmd_tick).cloned().collect()` → `take_for_tick(cmd_tick)`

### Relay 侧不修改

分析发现覆盖语义会导致空帧清空有效命令（drain 后下一帧发送空帧）。保持 `extend()` 原样。

## Risks / Trade-offs

- [Risk] ResMut 冲突 → 同一帧中其他系统也不同使用 ResMut<CommandBuffer>。simulation_driver_system 在 Update，network_flush_system 也在 Update，但共享同一 ResMut 会在 Bevy 中触发 panic。需要确认调度。

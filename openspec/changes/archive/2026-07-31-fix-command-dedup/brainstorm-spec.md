## Context

多人游戏中，移动指令（右键）需要多次点击才能生效，不稳定时灵时不灵。

根因：`network_flush_system` (transport.rs:137-142) 每帧使用 `iter().filter().cloned().collect()` 从 `cmd_buf` 读取命令，但命令残留不被移除。`player_sid` 每帧自增，relay 的 `(tick,player_id,sid)` 去重失效。relay 用 `extend()` 不断累加重复命令，finalize 后全部广播回来，导致同 tick 收到 N 条相同 MoveTo。

## Goals / Non-Goals

**Goals:**
- 消除命令重复发送，移动指令稳定响应
- 保证确定性不受影响

**Non-Goals:**
- 不修改 relay 协议
- 不修改 simulation 命令执行逻辑

## Decisions

### 方案 A：客户端 drain（实施）

Client side: `crates/bevy_adapter/src/transport.rs`
- `Res<CommandBuffer>` → `ResMut<CommandBuffer>`
- 替换 `iter().filter().cloned().collect()` 为 `take_for_tick(cmd_tick)`
- `take_for_tick` 使用 `drain` 取出即销毁

**Relay 侧未修改。** `extend()` 保持原样——覆盖语义会因空帧清空有效命令（`take_for_tick` drain 后下一帧发送空帧，覆盖会清除上一帧的有效指令）。

### 不选择方案 B 和 C 的排除理由

方案 B（仅 relay 去重）治标不治本。方案 C（A+B）被空帧覆盖问题否决。

## Risks / Trade-offs

- [Risk] `Res<CommandBuffer>` → `ResMut<CommandBuffer>` 需要检查系统中其他 Res 引用是否冲突 → [Mitigation] `CommandBuffer` 在同一帧中仅被 `network_flush_system` 和 `simulation_driver_system` 使用，两者走不同调度路径

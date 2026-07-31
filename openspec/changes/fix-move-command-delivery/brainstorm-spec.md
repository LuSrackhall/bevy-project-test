## Context

多人游戏中移动指令（右键）需要多次点击才能生效，时灵时不灵。

根因分析（通过双客户端 e2e 测试定位）：
1. 命令在 `command_issue_system` 中创建，目标 tick = `current_tick + 1`
2. `network_flush_system` 发送 `current_tick + 1` 的帧
3. relay 收到后 finalize 该 tick 并广播
4. **竞态**：若命令创建于 network_flush 之后（同一帧），或网络延迟导致命令晚到，relay 已 finalize 该 tick → 命令被丢弃
5. `input_delay` 字段从未被使用（本应缓冲命令指向未来 tick）

e2e 测试证据：双客户端锁步测试中，逐 tick 发命令 60/60 送达（本地无延迟）；但 `input_delay` 未使用导致真实网络下命令指向已 finalize 的 tick，被丢弃。

## Goals / Non-Goals

**Goals:**
- 命令指向未来 tick（`current_tick + input_delay`），relay 无法在其到达前 finalize
- 发送窗口帧 `[current_tick+1, current_tick+input_delay]` 保持 relay 顺序推进
- 输入系统先于 network_flush 运行（同帧内命令及时发送）
- 添加可靠性回归测试

**Non-Goals:**
- 不修改 relay 协议
- 不修改仿真命令执行逻辑

## Decisions

### 1. `SimulationDriver::command_delay()`

新增方法：Network 模式返回 `input_delay`，Live/Replay 返回 1。

### 2. 命令目标 tick

`command_issue_system` 和 `seek_stance_shortcut_system` 用 `current_tick + command_delay()` 替代 `current_tick + 1`。

### 3. network_flush_system 窗口发送

发送 `[current_tick+1, current_tick+input_delay]` 的每个 tick 一个帧，保持 relay 顺序 finalize。命令只在实际目标 tick（`current_tick+input_delay`）携带。

### 4. 系统排序

输入系统组 `.before(network_flush_system)`，确保同帧内命令先创建后发送。

### 5. e2e 可靠性测试

`network_move_e2e.rs`：双客户端，逐 tick 发 MoveTo，验证全部送达（不丢）。

## Risks / Trade-offs

- [Risk] input_delay=3 增加命令生效延迟 3 tick → 可接受（网络缓冲的必然代价，与 relay 配置一致）
- [Risk] 窗口发送增加每帧网络消息量 → 仅 3 个小帧，可忽略

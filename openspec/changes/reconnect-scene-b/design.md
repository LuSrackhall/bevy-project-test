## Context

Change 1-3 交付可靠 UDP + 重连分页后,重连仅支持 Scene A(进程存活)。本 change 实现 Scene B(进程重启):重启客户端重建世界 + 快速回放 + 续接。设计整合 3 子 agent 评审(C1 快速重启死锁、C2 席位判定、W3 回放速度、W4 map_size、W5 上发泄漏、W6 lobby 分支)。宪法约束:分层(§1)、确定性(§0.1)、同套仿真代码(§9)、NoOp 兜底(§3.2)。

## Goals / Non-Goals

**Goals:**
- 进程重启后重连运行中对局:重建(seed+map_size)→ 快速回放 → 续接实时
- Scene A 不回退;快速重启窗口被重试桥接
- 确定性(同 init + run_tick(enable_ai:false));防重复重建

**Non-Goals:**
- 加密 / wasm / 权威服务器;tick 管线与仿真改动;快照/存档

## Decisions

### 数据结构与协议

```rust
// network.rs — ReconnectResponse 增 map_size(Scene B 重建用,消除硬编码 Medium)
pub struct ReconnectResponse {
    // ...既有字段(seed, map_spec_hash, first_tick, total_ticks, page_count, players)
    pub map_size: simulation::map::MapSize,  // NEW
}
```

### relay 侧(relay_core.rs + network.rs)

1. **handle_reconnect**:响应增 `map_size`(与游戏配置一致)。`last_tick_consumed=0` 返回全量日志(Change 3 已支持)。
2. **JoinGame 补发 GameStarted(D2/C2)**:
   - `on_join_game` 复用 Disconnected 席位后,返回该玩家原本「是 Disconnected 席位」的信号。
   - relay_core JoinGame 分支:若 `server.is_game_started()` 且该加入复用 Disconnected 席位 → GameJoined 后 `broadcast` GameStarted(含 seed)。
   - 已 Playing 客户端收到重复 GameStarted 无害(lobby 系统仅 LobbyPhase 处理)。

### 客户端 transport(transport.rs)

1. **无条件 ReconnectRequest(D1/C1)**:
   - `udp_session` 总是发送 ReconnectRequest(last_tick_consumed = 当前值,全新进程 0),去掉 `> 0` 门控。
   - **有界重试**:`JoinRejected`/`Error` 一律 `return false`(重试),但 `spawn_network_client` 累计重试上限(如 10 次,指数退避 1s→30s)。桥接 relay 1.5s 心跳窗口。
2. **回放期上发门控(D6/W5)**:
   - `network_flush_system` 在 Scene B 追赶期间跳过(读取 driver 的 catch-up 状态);本地命令回放期丢弃,追平后恢复。

### driver 快速回放(driver.rs)(D5/W3)

- 增 `catch_up: Option<u32>`(剩余待追平 tick 数)状态。
- Scene B 重建后置位;每帧在 is_tick_ready 门控下批量执行 `min(剩余, BATCH)` 个 tick(复用 handle_seek 的批量 run_tick 逻辑,不把 clock 推到实时上限),直到 relay_buffer 耗尽/追平。
- 追平后清 catch_up,恢复 20Hz 实时。

### render_view lobby 过渡(W6)

- `lobby_update_system` 增 `NetworkEvent::Reconnect` 分支:用 ReconnectResponse 的 seed + map_size 设置 `NetworkGameStart` → 转 Playing(与 GameStarted 同路径)。
- `reset_game_system` 网络模式改用 `NetworkGameStart.map_size`(替代硬编码 Medium)。
- **防重复重建**:仅在 LobbyPhase 处理 Reconnect 分支触发重建;Playing 期间的 Reconnect 事件留给 reconnect_recovery_system(Scene A)。

### Scene A/B 判定(D3)

- 重连响应到达时的状态:
  - Playing(driver 运行)→ reconnect_recovery_system 处理(Scene A,现状)
  - Lobby(全新)→ lobby 系统处理(Scene B,重建)
- 边界:Playing 断在 tick0(世界存在)→ Scene A 不重建。

### 宪法落地

- 分层:bevy_adapter + render_view(既有 lobby/reset 路径);不动 simulation。
- 确定性:重建走同套 init + run_tick(enable_ai:false);seed/map_size 权威来自 relay;批量回放与逐 tick 同哈希。
- 幂等:仅 Lobby→Playing 重建;双 seed 一致性断言(GameStarted.seed == response.seed)。

## Risks / Trade-offs

- [快速重启死锁] → D1 有界重试桥接 1.5s 窗口
- [重建判定歧义] → D3 用 driver 生命周期状态 + 防重复重建
- [map_size desync] → D4 协议字段 + 一致性断言
- [回放慢] → D5 批量追赶
- [回放期上发泄漏] → D6 门控
- [hash 测试脆弱] → 真实命令(MoveTo)+ 确定性逐 tick 步进,禁空命令/float accumulator

## Migration Plan

1. 测试先行:handle_reconnect(last=0)+map_size 单测、relay 补发 GameStarted 单测、A/B 判定边界单测
2. 协议:ReconnectResponse 增 map_size
3. relay:JoinGame 补发 GameStarted
4. transport:无条件 ReconnectRequest + 有界重试 + 上发门控
5. render_view:lobby Reconnect 分支 + reset 用 map_size
6. driver:回放追赶模式
7. 集成:进程重启重建+快速回放 hash 全等、Scene A 回归、快速重启重试桥接
8. 回滚:分支级 revert(协议不兼容,两端同仓)

## Open Questions

- 有界重试上限/退避(10 次 / 1s→30s 初值)
- 回放 BATCH 大小(每帧 tick 数)
- 回放期本地输入丢弃 vs 缓冲

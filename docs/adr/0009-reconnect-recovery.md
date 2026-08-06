# ADR 0009: 掉线重连恢复语义（席位保留 + 重建路径统一）

## 状态

**Date**: 2026-08-05
**Status**: Accepted（实现于 fix-multiplayer-correctness）

## 背景

原实现掉线重连存在三个缺陷：

1. `on_disconnect` 把玩家从 `all_players` **永久剔除**，对局规模永久降为 N-1。
2. `next_player_id` 单调递增不复用 → 掉线重连被判 `Room is full`。
3. 客户端从未调用 `apply_reconnect`，重连链路未接通。

## 决策

1. **席位保留**：`on_disconnect` 不再剔除 `all_players`，改为记录 `Disconnected` 集合。重连时 `on_join_game` 优先复用断线席位（原 player_id）。
2. **Disconnected 席位放行**：`try_finalize` 的 all_ready 检查与 `game_started` 达成检查均排除 `Disconnected` 席位，靠超时兜底定稿，防止 barrier 挂起。
3. **客户端接线**：transport 在重连后发 `ReconnectRequest(last_tick_consumed)`；收到 `ReconnectResponse` 后，`reconnect_recovery_system` 调 `apply_reconnect` 灌入断点后日志，driver 复用现有 Network 管线续接（不另起 while 循环）。
4. **重建路径统一（R1）**：若需重建世界（进程重启场景），**必须**走 `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, local))` + `run_tick(enable_ai:false)`，与正常网络路径完全一致。`init_simulation_world`（2 槽）/ `run_tick_default`（AI 开）被禁止——它们与网络 PlayerSlots/NoOp 集合不一致，必然 desync。

## 场景划分

- **场景 A（网络断开，进程存活）**：本地世界停在断点，无需重建，只需 `apply_reconnect` 灌断点后日志，driver 从断点续接。本次实现的主路径。
- **场景 B（进程重启）**：需重建世界，走 R1 路径（`rebuild_world` 封装在 bevy_adapter 会话层）。为后续扩展预留。

## 影响

- 掉线玩家席位保留，重连复用原 player_id（`test_disconnect_retains_seat_and_reconnect_reuses_id`）。
- Disconnected 席位不挂起 barrier（`test_disconnected_player_does_not_hang_barrier`）。
- `apply_reconnect` 灌日志 + 版本校验（`test_apply_reconnect_loads_log_and_validates_version`）。
- 世界重建逻辑移入 bevy_adapter（消除 render_view 直触仿真，分层修正）。

## 关联

- specs/network-reconnect、specs/relay-server（delta specs）
- 宪法 §3.1/§3.2（NoOp 补齐）、§9.1（同套仿真代码）、§5.5（分层）

## Context

当前联机硬编码 2 人 TCP lockstep,relay 内嵌为主(`session_host`),目标规模 8 人以上。代码中存在大量 ≤8 / 2 人的硬编码与隐性假设,阻止对局规模参数化。经双 agent 交叉审核(main=4e57016 上 7 个修改点全部仍有效)与宪法合规审核(无违反,含 4 项强制修正已并入),本 change 只做**与传输无关的正确性修复**;传输层(UDP + 内存通道)是独立后续 change。

## Goals

- 解除所有 ≤8 / 2 人的硬编码与隐性假设,让玩家数 N 成为配置参数且逻辑正确
- 掉线重连完整可用:席位回收 + 日志重放恢复,且重放路径与正常网络路径初始化**完全一致**(防 desync)
- UI 创建房间人数可选 2..=8
- 单机 2 人行为与全部确定性测试保持
- 满足宪法:补锁步回归测试 + ADR

## Non-Goals

- 传输层改造(独立后续 change:自写可靠 UDP + 内嵌内存通道)
- web/wasm 联机、队伍模式、AI 泛化、IPv6/P2P、带宽优化

## Decisions

### D1 — 解除硬编码(4 项,均不引入仿真禁区概念)

1. `lobby_ready_mask: u8`(bevy_adapter/src/network.rs:352,421-429)→ `HashSet<u8>`,ready 判定 `set.len() >= all_players.len()`。解除 8 人硬上限。
2. `PlayerSlots::multi_player` 的 `assert!(count <= 8)`(simulation/src/types.rs:358)移除,count 由 `u8` 类型上限(255)兜底。
3. **city_capture**(simulation/src/soldier/mod.rs:859-864):城市 HP=0 时归 `last_attacker_faction`,替换 0↔1 阵营互换 + `FactionId(2)` 中立。单机 2 人下行为等价(攻击者只能是对方),多人下才正确。`last_attacker_faction` 由确定性 combat 写入(mod.rs:989)且已入 golden hash 覆盖(golden_test.rs:113)。
4. `collect_command_players` 兜底 `if id <= 1`(simulation/src/lib.rs:103)移除。

### D1b — UI 参数化

- `max_players: 2` 默认值(render_view/src/lib.rs:672)与按钮文本"2"无 cycle 处理器(lan_lobby.rs:317,408)→ 参数化 + cycle 到 2..=8。
- `current_players` 随加入/离开正确更新(当前恒 1,lib.rs:707)。

### D2 — 掉线重连(完整,含宪法强制修正)

1. `on_disconnect`(network.rs:595-598)不再永久剔除 `all_players`,标记 `Disconnected` 保留席位。
2. `next_player_id` 单调递增(network.rs:354,408-413)→ 分配"首个空闲席位",重连复用原 player_id(修复"重连被拒 Room is full")。
3. **R1(强制)** 客户端接上 `apply_reconnect`(network.rs:280,目前零调用点):收到 `ReconnectResponse` 后,世界重建**必须**走 `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, self))` + 原地图 + `run_tick(enable_ai:false)`,与正常网络路径(render_view/lib.rs:490-497 + driver.rs:347-355)完全一致。**禁止** `init_simulation_world`/`run_tick_default`(现有 network.rs:273-274 注释即错,一并修正)。
4. **R2(推荐)** 重连快进复用 driver 的 Network 管线(`apply_reconnect` 灌 relay_buffer → `simulation_driver_system` 消费),勿另起 while 循环。
5. **R3(强制)** `try_finalize` 的 all_ready 检查(network.rs:506-510)对 `Disconnected` 席位放行,靠超时兜底定稿,防止永久挂起。
6. **R4(强制)** 重连地图一致性:消除网络地图硬编码 `MapSize::Medium`(render_view/lib.rs:462),建立 `map_spec_hash` → MapSize 映射,重连地图与对局一致。
7. **分层修正** 世界重建逻辑移入 bevy_adapter 通道:render_view/lib.rs:494 现直接调 `init_simulation_world_multi`,属下游直触仿真,违反分层拓扑(§1.1/§5.5),随本变更修正。

### 合规补齐(宪法强制)

- **§10.1 锁步回归测试**:① city_capture 多人归属确定性测试(含单机 2 人行为等价断言);② 重连重放确定性测试(重建路径 vs 正常路径世界 hash 一致)。
- **§16 ADR**:城市归属语义变更 + 重连恢复语义,各一份 ADR。

## Risks / Trade-offs

- [城市归属语义改动破坏单机确定性] → 单机 2 人行为等价断言 + golden test 覆盖
- [重连重建 desync] → R1 强制统一初始化路径(init_simulation_world_multi + run_tick(enable_ai:false))+ 锁步回归测试兜底
- [席位模型改动波及满员判定] → 集成测试覆盖"8 人满员拒绝 / 掉线后重连复用"
- [断线玩家的 NoOp 补齐依赖超时] → R3 放行 Disconnected 席位 + 既有超时兜底逻辑

## 后续(独立 change,不在本次)

- Change 2:传输层自写可靠 UDP + 内嵌内存通道,消除 TCP 队头阻塞
- Change 3+:web/wasm、带宽优化、队伍模式、AI 泛化、IPv6/P2P

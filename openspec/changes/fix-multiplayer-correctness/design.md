## Context

Change 1 目标:解除 8 人以上联机的正确性障碍(硬编码 + 掉线重连未接通)。高层设计已由 brainstorm-spec.md 批准(Goals/Decisions/Risks)。本文深入实现架构、数据流、模块边界、错误处理与测试策略,聚焦"为什么这样实现"。宪法约束:分层拓扑、simulation 白名单、确定性、命令驱动、同套仿真代码(§1/§2/§3/§9)。

## Goals / Non-Goals

**Goals:**
- 玩家数 N 成为配置参数,解除 ≤8/2 硬编码,逻辑在 N>2 下正确
- 掉线重连完整可用,且重连重建路径与正常网络路径 bitwise 一致(防 desync)
- 满足宪法:锁步回归测试 + ADR

**Non-Goals:**
- 传输层(UDP/内存通道)留待 Change 2
- web/wasm、队伍、AI 泛化、带宽优化

## Decisions

### D1: ready 掩码结构 — `HashSet<u8>`(非扩大位掩码)

`lobby_ready_mask: u8`(network.rs:352)改为 `lobby_ready: HashSet<u8>`。ready 判定 `lobby_ready.len() >= active_players.len()`。

**Alternatives**: 位掩码扩到 u64 到 64 人仍有上限且位运算不直观;`Vec<bool>` 语义较弱。HashSet 语义清晰、无实际上限(受 `u8` player_id 约束),改动最小。

### D2: 席位模型 — 保留席位标记 `Disconnected`

`on_disconnect`(network.rs:595-598)不再 `all_players.retain(...)`,改为将玩家从 `ready` 集合移除并保留 `Disconnected` 标记。`PlayerState` 枚举已含 `Disconnected`(network.rs:17-19)。重连时复用原 `player_id`。

`next_player_id` 分配(408-413)改为"首个空闲席位":`0..all_players.len()` 中第一个非活跃 player_id。

**Alternatives**: 掉线即移除、重连重新加入 —— 无法保证重连后拿到原 player_id,且 `try_finalize` 的 NoOp 注入集合会漂移。保留席位保证 player_id 稳定。

### D3: 重连路径 — 复用 driver Network 管线(不另起 while 循环)

客户端接通 `apply_reconnect`(network.rs:280,现零调用点):收到 `ReconnectResponse` 后
1. `apply_reconnect` 校验 `ruleset_version` 并把 `ticks` 灌入 `relay_buffer`
2. 世界重建走 `init_simulation_world_multi(seed, PlayerSlots::multi_player(N, self))` + 原地图
3. 恢复后 `simulation_driver_system` 依 `is_tick_ready` 顺序消费 relay_buffer 重放到断点,再正常 lockstep

**R1 强制**:重建**禁止** `init_simulation_world`/`run_tick_default`(现有 network.rs:273-274 注释即错,一并修正)。`run_tick(enable_ai:false)` 与正常网络路径一致(driver.rs:347-355)。

**Alternatives**: 独立 while 循环快进 —— 绕过 driver 管线,易引入与正常路径的 AI/NoOp 差异。复用管线保证同一仿真入口(宪法 §9.1)。

### D4: 地图一致性 — `map_spec_hash` → MapSize 映射

消除 render_view/lib.rs:462 的 `Network => MapSize::Medium` 硬编码。`ReconnectResponse` 携带 `map_spec_hash`(network.rs:117)。实现时建立确定性 `map_spec_hash → MapSize` 映射(生成地图时计算 hash),重连解析回同一 MapSize。

**Alternatives**: 协议新增 MapSize 字段 —— 协议面更宽;映射改动最小且 hash 已在协议中。

### D5: 分层 — 世界重建封装在 bevy_adapter

render_view/lib.rs:494 直调 `init_simulation_world_multi` 违反分层拓扑(§1.1/§5.5)。世界重建逻辑封装为 bevy_adapter 会话层函数(如 `session::reconnect::rebuild_world`),render_view 只驱动流程,不直触仿真。重连重建与此共用同一函数。

### D6: city_capture 归属 — 归 `last_attacker_faction`

soldier/mod.rs:859-864 的 0↔1 互换改为 `city.last_attacker_faction.unwrap_or(FactionId(0))` 语义(城市 HP=0 时归最后攻击者;无攻击者保持现状兜底)。`last_attacker_faction` 由确定性 combat 写入(mod.rs:989)且已入 golden hash(golden_test.rs:113),单机行为等价。

**注意**: 兜底值需确认。现 `FactionId(2)` 分支已用 `last_attacker_faction.unwrap_or(FactionId(0))`,扩展为所有 faction 统一此语义。

## Risks / Trade-offs

- [城市归属语义改变] → 单机 2 人等价断言(攻击者只能是对方)+ golden test
- [重连重建 desync] → R1 统一初始化路径 + 重连重放确定性测试(重建 vs 正常路径 hash 一致)
- [席位模型改挂起 tick barrier] → R3:`try_finalize` all_ready 对 Disconnected 放行,靠超时兜底定稿
- [map_spec_hash 映射未定义] → 实现时确定 hash 算法,测试覆盖同一地图 hash 往返

## Migration Plan

1. 先加确定性测试(城市捕获多人 + 重连重放 hash 一致),再改实现 —— 宪法 §10.1
2. 实现顺序:D1(掩码)→ D6(city_capture)→ D2(席位)→ D3/D4/D5(重连链路)→ D1b(UI)
3. 回滚:分支级 revert;改动均在独立分支,不污染 main

## Open Questions

- `map_spec_hash` 的具体哈希算法与 `MapSize` 的确定性映射(实现时确认,测试覆盖)
- `apply_reconnect` 接线点:transport.rs 收到 `ReconnectResponse` 后的触发路径(现仅 eprintln,472-474)

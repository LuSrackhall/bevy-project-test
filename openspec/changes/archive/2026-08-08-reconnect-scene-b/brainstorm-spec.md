## Context

Change 1-3 已交付多人规模、正确性、可靠 UDP 传输、重连日志分页。**重连恢复目前只支持 Scene A(网络断开,进程存活)**:`apply_reconnect` 把分页日志灌入 relay_buffer,driver 从断点续放。**Scene B(进程重启)不可恢复**:
- 客户端 transport.rs:234 仅当 `last_tick_consumed > 0` 才发 ReconnectRequest → 全新进程(last=0)不请求日志
- relay 对重连者不重发 GameStarted → lobby 系统不转 Playing
- 无世界重建路径(reset_game_system 用 GameStarted 的 seed,Scene B 无 GameStarted)
- 席位保留已实现(Change 1,Disconnected 座位原 id 复用),但新进程无法续玩

经 3 子 agent 多维度审查(代码架构/宪法合规/测试策略),设计整合全部关键发现:
- CRITICAL C1:无条件 ReconnectRequest 使全新进程在快速重启窗口(relay 心跳 1.5s 清理旧席位)内 JoinGame 命中 Room is full,且按 `last_tick_consumed()==0` 放弃 → 线程永久退出,Scene B 死锁。须有界重试桥接。
- CRITICAL C2:on_join_game 按 `disconnected.iter().min()` 复用席位,无法区分「重连」与「新玩家接管」;窗口内甚至复用不了。须先解 C1,复用成功即重连信号。
- W3 回放速度:20Hz 逐 tick 追平数千 tick 需数分钟,须批量追赶。
- W4 map_size:ReconnectResponse 无 map_size,reset_game_system 硬编码 Medium,非 Medium 地图必 desync。须入协议。
- W5 回放期上发泄漏:回放中 network_flush_system 上发已定稿 tick 帧,relay staging 泄漏。须门控。
- 宪法:分层合规;须防重复重建(仅 Lobby→Playing 可重建);双 seed 定单一权威。

## Goals / Non-Goals

**Goals:**
- 进程重启后可重连运行中的对局:重建世界(seed+map_size)→ 快速回放全量日志 → 续接实时
- 复用 Change 3 分页(元数据补 map_size 字段)
- 保持确定性:同 init + run_tick(enable_ai:false)(宪法 §9 同套仿真代码)
- Scene A 路径不回退;快速重启窗口可被重试桥接

**Non-Goals:**
- 加密 / wasm / 权威服务器(后续)
- 改动 tick 管线与仿真本体
- 快照/存档(重放从 seed 重建,等价 replay 语义)

## Decisions

### D1: 无条件 ReconnectRequest + 有界重试(C1 修复)

- transport.rs `udp_session`:**总是**发送 ReconnectRequest(last_tick_consumed = 当前值,全新进程为 0)。
- **C1 修复**:JoinRejected/Error 不再按 `last_tick_consumed()==0` 放弃;`spawn_network_client` 对二者一律重试,但**总重试上限**(如 10 次,指数退避 1s→30s)。快速重启窗口(1.5s)被重试桥接,旧席位变 Disconnected 后 JoinGame 复用成功。
- 全新对局下 last=0 的 ReconnectRequest → relay 返回 total_ticks=0 → 客户端立即完成,无副作用(回归测试)。

### D2: relay 复用 Disconnected 席位时补发 GameStarted(C2 修复)

- C1 修复后重连者最终拿到 Disconnected 席位(on_join_game 按 `disconnected.iter().min()` 复用原 player_id)。
- relay_core JoinGame 分支:若 `server.is_game_started()` **且**该客户端复用 Disconnected 席位(即重连),响应 GameJoined 后**再发一次 GameStarted**(含 seed)。
- 已 Playing 客户端收到重复 GameStarted 无害(lobby 系统仅 LobbyPhase 处理,Playing 下被 `_ => {}` 丢弃)。

### D3: 客户端 Scene A/B 判定 + Scene B 流程

- **场景判定**(测试评审 #5):用 driver 生命周期状态,不用 last_tick_consumed:
  - 已 Playing(driver 运行中,世界存在)→ Scene A:apply 页到 relay_buffer,driver 续放(现状)
  - Lobby/全新(driver 未起步,无世界)→ Scene B:重建 + 回放
  - 边界:Playing 但断在 tick0(世界存在、driver=0、last=0)→ **Scene A 不重建**;全新进程(无世界)→ Scene B 重建
- **Scene B 流程**:
  1. 收到 ReconnectResponse 元数据(seed + map_size + players)+ 补发 GameStarted
  2. lobby 系统新增 ReconnectResponse 分支:用重连元数据的 seed/map_size 设置 NetworkGameStart → 转 Playing(与 GameStarted 同路径)
  3. reset_game_system 用 `rebuild_world(seed, player_count, player_id)` + `generate_map(map_size)` 建世界(player_count 取 GameJoined/GameStarted,非回退 2-slot)
  4. 页面逐页灌入 relay_buffer;driver 从 tick 0 起步,is_tick_ready 门控
  5. 快速回放(D5)追平后接实时 broadcast
- **防重复重建(宪法警告)**:仅 Lobby→Playing 过渡可重建;Playing 期间的重连响应一律走 Scene A apply,不得再次触发 reset。

### D4: 协议增字段 — map_size(W4 修复)

- `ReconnectResponse` 增 `map_size: simulation::map::MapSize`。
- relay handle_reconnect 携带(与游戏配置一致);客户端 rebuild 用 `generate_map(map_size)`,消除硬编码 Medium 的 desync 隐患。
- **双 seed 单一权威**:Scene B 重建只用 ReconnectResponse 的 seed;`GameStarted.seed == response.seed` 一致性断言(不等则日志告警)。

### D5: 快速回放模式(W3 修复)

- 纯 20Hz 逐 tick 回放对长断线(数千 tick)太慢。
- 新增**回放追赶模式**:Scene B 重建后、追平前,driver 每帧执行 N 个 tick(复用 handle_seek 的批量 run_tick 逻辑,dry-run 不把 clock 推到超限),直到 relay_buffer 耗尽/追平,再切回 20Hz 实时。
- 确定性:批量 run_tick 与逐 tick 同哈希(handle_seek 已证明批量等价)。

### D6: 回放期上发门控(W5 修复)

- 回放期间 `network_flush_system` 为 current+1..+3 上发 PlayerTick 帧,已定稿 tick 的帧在 relay staging 泄漏。
- 修复:Scene B 回放追赶期间门控本地输入(network_flush_system 跳过);本地命令在回放期丢弃/缓冲,追平后恢复。

### D7: 层约束(宪法评审确认合规)

- 改动在 bevy_adapter(transport/relay_core/network/driver)+ render_view(lobby 过渡,既有路径)。
- lobby 系统写 render_view 本地资源 NetworkGameStart(非 simulation 命令),触发既有 OnEnter(Playing)→ reset_game_system。无 render_view 直写 simulation;回放走 driver 唯一 run_tick 入口(§9)。

### 宪法约束

- 分层:不动 simulation;rebuild_world 在 bevy_adapter 会话层(已有原语)。
- 确定性:重建用同套 init+run_tick,页面是确定性日志子集;map_size/seed 权威来自 relay。
- 幂等:仅 Lobby→Playing 可重建;GameStarted 重发无害。
- 文档:specs/network-reconnect 更新 Scene B 为已实现;ADR-0009 补记 GameStarted 重发协议变更。

## Risks / Trade-offs

- [快速重启死锁] → D1 有界重试桥接 1.5s 窗口
- [重建判定歧义] → D3 用 driver 生命周期状态 + 防重复重建
- [map_size 缺失 desync] → D4 元数据补字段 + 一致性断言
- [回放慢] → D5 批量追赶
- [回放期上发泄漏] → D6 门控
- [跨层耦合] → D7 只经既有 NetworkEvent + NetworkGameStart 通道
- [长断线回放 hash 不可用空命令/float] → 测试用 MoveTo 真实命令 + 确定性逐 tick 步进(测试评审)

## Migration Plan

1. 测试先行:handle_reconnect(last=0)+map_size 单测;relay 补发 GameStarted 单测;A/B 判定边界单测
2. 协议:ReconnectResponse 增 map_size
3. relay:JoinGame 复用 Disconnected 席位补发 GameStarted
4. 客户端 transport:无条件 ReconnectRequest + 有界重试
5. render_view lobby:ReconnectResponse 分支 → NetworkGameStart + 转 Playing
6. driver:回放追赶模式;network_flush_system 回放期门控
7. 集成:进程重启模拟重建+快速回放 hash 全等(真实命令 MoveTo + 确定性步进);Scene A 回归;快速重启重试桥接
8. 回滚:分支级 revert

## Open Questions

- 有界重试上限与退避参数(10 次 / 1s→30s 初值)
- 回放追赶 BATCH 大小(每帧 tick 数,与 UI 帧率权衡)
- 回放期本地输入丢弃 vs 缓冲(丢弃更简单,缓冲可恢复本地意图)

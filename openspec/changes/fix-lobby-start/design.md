## Context

relay 在最后一名玩家就绪时于 `handle_message(LobbyReady)` 内背靠背广播 `LobbyUpdate` → `GameStarted`(relay_core.rs)。两者经可靠 Control 通道先后到达客户端,`NetworkEventReceiver.drain_all()` 在单帧内一并消费。`lobby_update_system` Connected 阶段按序迭代,就绪 `LobbyUpdate` 触发 `state.phase = Ready; return`,同批 `GameStarted` 被丢弃 → 客户端永久卡在大厅(无法开局)。

## Goals / Non-Goals

**Goals:**
- 同批 `LobbyUpdate`(本机就绪)+ `GameStarted` 时客户端正确进入 Playing
- 仅 `LobbyUpdate` 时保持 Ready 语义,由后续批次 GameStarted 接手
- 回归测试锁定同批与分批两种场景;transport 层两条开局路径锚定

**Non-Goals:**
- 不改 relay/transport(已验证正确)
- 不改 Ready 阶段处理(无同批问题)

## Decisions

**1. 移除 Connected 阶段就绪分支的 `return`** — 置 `phase = Ready` 后继续迭代本批。同批 `GameStarted` 命中其 handler:`received = true` + `next_state.set(Playing)`。无 GameStarted 时循环自然结束,Ready 语义不变。该 `return` 仅提前退出迭代,无其他副作用。

**2. 事件顺序保证** — 可靠 Control 通道保证 `LobbyUpdate` 先于 `GameStarted` 到达,故同批内后者总是可被本次修复接住。若未来 relay 广播顺序变化,Ready 阶段仍兜底(逐帧查 GameStarted)。

**3. 测试分层** — 单元级(lobby_system.rs)直接构造事件批验证丢弃/不丢弃;集成级(lobby_start_e2e.rs)经真实 transport 验证两条开局路径送达 GameStarted。

## Risks / Trade-offs

- [同批后续事件类型不可预期] → if-chain 仅 GameStarted 触发状态转换,其余忽略,无风险
- [未来 relay 只广播 GameStarted] → Ready 阶段路径仍正确,修复无副作用

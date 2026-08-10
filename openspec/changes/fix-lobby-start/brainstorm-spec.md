## Context

联机开局流程存在确认缺陷:当最后一名玩家就绪时,relay 在 `handle_message(LobbyReady)` 内**背靠背广播两条消息**——`LobbyUpdate`(就绪状态)→ `GameStarted`(开局)。两者落在同一个可靠 Control 通道,客户端在同一帧 `NetworkEventReceiver.drain_all()` 批次里一起收到。

render_view 的 `lobby_update_system` Connected 阶段按序处理事件:
```rust
if let NetworkEvent::LobbyUpdate { players, .. } = event {
    let local_ready = players.iter().any(|p| p.player_id == network_start.player_id && p.ready);
    if local_ready {
        state.phase = LobbyPhase::Ready;
        return;   // ← BUG: 同批后续的 GameStarted 被丢弃
    }
}
```
就绪的 `LobbyUpdate` 触发 `return`,同批的 `GameStarted` 被消费并丢弃 → 客户端 phase 变 Ready,但 GameStarted 永不再次到达 → **永久卡在大厅,无法开局**。

用户实测:Win 创建房间 + Mac 加入后无法开局。transport 层已由 e2e 测试(`lobby_start_e2e.rs`)证明两条开局路径(自动开局/就绪开局)均能送达 GameStarted,缺陷定位在 render_view 的批处理逻辑。

## Goals / Non-Goals

**Goals:**
- 同批收到 `LobbyUpdate`(本机已就绪)+ `GameStarted` 时,客户端正确进入 Playing,不丢 GameStarted
- Ready 语义保持:仅 `LobbyUpdate` 无 `GameStarted` 时仍进 Ready 阶段,由后续帧的 GameStarted 接手
- 回归测试覆盖:同批丢弃场景 + 分批场景

**Non-Goals:**
- 不改 transport / relay 层(已验证正确)
- 不改 Ready 阶段的事件处理(其只查 GameStarted,无同批丢弃问题)
- 不重构 `lobby_update_system` 整体结构

## Decisions

**1. 修复方式:去掉 Connected 阶段就绪分支的 `return`** — 设置 `state.phase = LobbyPhase::Ready` 后继续迭代本批事件,使同批后续的 `GameStarted` 仍被处理(置 `received`、`next_state.set(Playing)`)。若本批无 `GameStarted`,循环自然结束,phase 已是 Ready,下一帧由 Ready 阶段接手——语义不变。之所以可行:该 `return` 的唯一作用是提前退出迭代,去掉后对仅含 LobbyUpdate 的批次无副作用。

**2. 测试** — 两个层级:
- `crates/render_view/tests/lobby_system.rs`(单元级):构造最小 Bevy App,喂入同批 `[LobbyUpdate(本机就绪), GameStarted]`,断言 `NetworkGameStart.received == true`;以及仅 `[LobbyUpdate]` 后分帧 `GameStarted` 仍能开局。
- `crates/bevy_adapter/tests/lobby_start_e2e.rs`(集成级):真实 transport 双客户端,验证自动开局与就绪开局两条路径均送达 GameStarted(transport 层无回归锚点)。

## Risks / Trade-offs

- [若未来 relay 改为只广播 GameStarted 不广播 LobbyUpdate] → 本修复无副作用,Ready 阶段路径仍正确
- [去掉 return 后同批后续事件可能包含非预期消息] → 事件由 if-chain 过滤,仅 GameStarted 触发状态转换,其余被忽略,无风险

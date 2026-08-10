## Why

联机开局缺陷:最后一名玩家就绪时,relay 背靠背广播 `LobbyUpdate` → `GameStarted`,两者落入客户端同一事件批次。`lobby_update_system` Connected 阶段处理就绪的 `LobbyUpdate` 后提前 `return`,同批的 `GameStarted` 被丢弃,客户端永久卡在大厅——Win 建房间 + Mac 加入后无法开局。transport/relay 层已验证正确,缺陷在客户端批处理逻辑。

## What Changes

- `crates/render_view/src/lib.rs` `lobby_update_system` Connected 阶段:移除就绪 `LobbyUpdate` 分支后的 `return`,使同批后续事件(尤其 `GameStarted`)继续被处理;语义不变——仅含 LobbyUpdate 的批次仍进 Ready 阶段。
- 新增回归测试:
  - `crates/render_view/tests/lobby_system.rs`:同批丢弃场景 + 分批场景
  - `crates/bevy_adapter/tests/lobby_start_e2e.rs`:真实 transport 双客户端,自动开局 / 就绪开局两条路径均送达 GameStarted

## Capabilities

### New Capabilities
<!-- 无新能力 -->

### Modified Capabilities
- `relay-lobby-protocol`: 新增/修改要求——客户端在收到就绪 `LobbyUpdate` 的同批中收到 `GameStarted` 时,必须完成开局转换(不得丢弃)。

## Impact

- `crates/render_view/src/lib.rs`(lobby_update_system,1 行改动)
- `crates/render_view/tests/lobby_system.rs`(新增)
- `crates/bevy_adapter/tests/lobby_start_e2e.rs`(新增)
- 规格 `openspec/specs/relay-lobby-protocol/spec.md`(要求变更)

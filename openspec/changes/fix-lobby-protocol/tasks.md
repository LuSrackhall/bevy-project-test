## 1. 修复 JoinGame 发送

- [x] 1.1 `spawn_network_client` / `spawn_network_client_nonblocking` 增加 `relay_id: RelayId` 参数
- [x] 1.2 在 TCP 连接后、`run_session` 前，发送 `RelayClientMessage::JoinGame`
- [x] 1.3 编译验证

## 2. 扩展 NeedsGameReset + 引入 IsHost

- [x] 2.1 `NeedsGameReset::Network` 增加 `relay_id: RelayId` 字段，修复所有 match 分支
- [x] 2.2 新增 `IsHost(bool)` Resource
- [x] 2.3 新增 `LobbyPlayerList` Resource
- [x] 2.4 `handle_join_room` 适配新字段 + 注入 IsHost(false)

## 3. 房主进入 Lobby

- [x] 3.1 `handle_create_room` 成功后：获取 endpoint + relay_id → 设置 NeedsGameReset::Network → 注入 IsHost(true) → GameState::Lobby
- [x] 3.2 `setup_lobby_system` 传递 relay_id 给 spawn_network_client_nonblocking

## 4. 修复 LobbyUpdate

- [x] 4.1 `lobby_update_system` 收到 LobbyUpdate 后存入 `LobbyPlayerList`，按本地玩家 ready 状态设 LobbyPhase
- [x] 4.2 编译 + 测试验证（`cargo build` + `cargo test -p bevy_adapter --lib`）

---

## Post-Implementation Workflow

<!-- DO NOT MODIFY THIS SECTION — it defines the required workflow after all tasks are complete -->

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

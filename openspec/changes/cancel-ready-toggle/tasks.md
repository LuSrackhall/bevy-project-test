## 1. 协议扩展（RelayServer + relay_core）

- [x] 1.1 `network.rs`: RelayServer 新增 `on_lobby_not_ready(player_id)` + 公开 `is_game_started()`
- [x] 1.2 `relay_core.rs`: LobbyReady 处理添加 game_started 守卫 + 双向分发 + 统一 LobbyUpdate

## 2. 客户端适配（transport + lobby UI）

- [x] 2.1 `transport.rs`: `send_lobby_ready(player_id, ready: bool)` 参数化，更新所有调用处
- [x] 2.2 `lobby.rs`: 就绪按钮 observer toggle ReadyState + `update_ready_button` 双向更新

## 3. 验证

- [ ] 3.1 编译 + 测试（`cargo build` + `cargo test -p bevy_adapter --lib`）

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

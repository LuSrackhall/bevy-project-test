## 1. 玩家列表容器 + update_lobby_player_list

- [x] 1.1 `setup_lobby_ui` 增加 `LobbyPlayerListContainer` 节点（flex_grow 容器）
- [x] 1.2 新增 `update_lobby_player_list` 系统：读取 LobbyPlayerList → despawn 旧行 → spawn 新行
- [x] 1.3 注册到 `mod.rs`：`.after(lobby_update_system).run_if(in_state(GameState::Lobby))`

## 2. 就绪/开始按钮

- [x] 2.1 非房主：就绪按钮点击发送 LobbyReady(true)，按钮文本改为"已就绪"并禁用
- [x] 2.2 房主：显示"开始游戏"按钮，点击发送 LobbyReady(true)
- [x] 2.3 编译 + 测试验证

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

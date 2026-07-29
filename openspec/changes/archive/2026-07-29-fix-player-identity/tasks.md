## 1. 修复 player_id

- [x] 1.1 `lobby_update_system` GameJoined handler 加 `network_start.player_id = *player_id`
- [x] 1.2 `lobby_update_system` GameJoined handler 加 `network_start.player_count = *player_count`

## 2. 修复 max_players

- [x] 2.1 `JoinRoomRequest` 新增 `max_players: u8` 字段及 Default
- [x] 2.2 `handle_join_room` 用 `request.max_players` 替代 `let max_players = 2u8`
- [x] 2.3 observer 加 `request.max_players = pkt_for_join.advertisement.room.max_players`

## 3. 测试与验证

- [x] 3.1 更新 `test_handle_join_room_sets_network_reset` 断言
- [x] 3.2 `cargo test -p render_view` 全部通过
- [x] 3.3 `cargo check` 全部通过

---

## Post-Implementation Workflow

<!-- DO NOT MODIFY THIS SECTION -->

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

## 1. 协议扩展

- [x] 1.1 扩展 `RelayClientMessage::JoinGame` 添加 `room_id: RoomId` 和 `relay_id: RelayId` 字段
- [x] 1.2 新增 `RelayServerMessage::JoinRejected { reason: String }`
- [x] 1.3 `RelayServerMessage::GameJoined` 增加 `player_count: u8` 字段

## 2. Relay JoinGame 处理器

- [x] 2.1 在 `RelayServer` 中实现 `on_join_game` 方法：验证 `relay_id`、分配 slot、返回 `GameJoined` 或 `JoinRejected`
- [x] 2.2 在 `relay/src/lib.rs` 的 `handle` 函数中将 `JoinGame` 从空实现改为调用 `on_join_game`
- [x] 2.3 满员拒绝：`current_players >= max_players` → `JoinRejected`

## 3. LocalPlayerIdentity Resource

- [ ] 3.1 在 `render_view/src/lib.rs` 中定义 `LocalPlayerIdentity { player_id, player_count }` Resource
- [ ] 3.2 在收到 `NetworkEvent::GameJoined` 时写入 `LocalPlayerIdentity`
- [ ] 3.3 `NetworkCommandSource` 创建时从 `LocalPlayerIdentity` 读取 `player_id`

## 4. NeedsGameReset.Network 简化

- [ ] 4.1 移除 `player_id` 字段，只保留 `relay_addr` 和 `player_count`
- [ ] 4.2 更新 `src/main.rs` 中 `--relay` CLI 路径
- [ ] 4.3 更新所有引用 `NeedsGameReset::Network` 的代码（setup_lobby_system 等）

## 5. JoinRoomRequest + Integration System

- [ ] 5.1 定义 `JoinRoomRequest { requested, room_id, relay_id, endpoint }` Resource
- [ ] 5.2 实现 Join Integration System：读取 `JoinRoomRequest` → TCP 连接 → 发送 `JoinGame`
- [ ] 5.3 在 `render_view` 插件中注册 Resource 和 System
- [ ] 5.4 处理 `JoinRejected`（打日志、清理连接状态）

## 6. 测试

- [ ] 6.1 relay 集成测试：`JoinGame` → `GameJoined`
- [ ] 6.2 relay 集成测试：满员 → `JoinRejected`
- [ ] 6.3 relay 集成测试：relay_id 不匹配 → `JoinRejected`

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

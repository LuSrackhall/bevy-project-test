## Context

多人游戏中，创建者和加入者都操控同一阵营（player_id 均为 0）。

根因：`handle_join_room` 设置 `NeedsGameReset::Network { player_id: None }`，
`setup_lobby_system` 将其转为 `network_start.player_id = 0`（临时值）。
但当 `GameJoined` 事件到达（relay 分配真实验 player_id，如 1），
`lobby_update_system` 更新了 `NetworkCommandSource` 和 `LocalPlayerIdentity`，
却没有同步更新 `network_start.player_id`。`reset_game_system` 读取 `network_start`
创建 `LocalPlayerId(0)`，导致双方使用同一 player_id。

## Goals / Non-Goals

**Goals:**
- 加入者获得正确的 player_id（由 relay 分配）
- 去掉 `handle_join_room` 中 `max_players` 的硬编码
- 上述修复均有自动化测试覆盖

**Non-Goals:**
- 不重构整套 player identity 架构
- 不修改 relay 层

## Decisions

### 修复集

1. **GameJoined handler** 加两行：
   `network_start.player_id = *player_id;`
   `network_start.player_count = *player_count;`

2. **`JoinRoomRequest`** 新增 `max_players: u8` 字段

3. **`handle_join_room`** 用 `request.max_players` 替代 `let max_players = 2u8`

4. **lan_lobby.rs observer** 加 `request.max_players = pkt_for_join.advertisement.room.max_players`

5. **测试更新**：修复 `test_handle_join_room_sets_network_reset` 断言；可选项新增 `lobby_update_system` 的 GameJoined 测试

## Risks / Trade-offs

- [Risk] setup_lobby_system 临时 `player_id = 0` → GameJoined 到达前有窗口期 → [Mitigation] 极短（TCP 首条消息即为 GameJoined），无实际影响

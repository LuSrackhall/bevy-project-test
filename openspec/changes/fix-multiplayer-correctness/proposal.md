## Why

当前联机硬编码 2 人(relay 默认、UI `max_players: 2`、simulation 阵营 0↔1 互换),并有多处 8 人硬上限(`lobby_ready_mask: u8`、`PlayerSlots` assert)。目标规模是 8 人以上,这些正确性障碍会随玩家数增长直接失效或产生 desync。同时掉线重连链路(席位回收、客户端重建世界)从未接通——8 人局掉线概率远高于 2 人局,必须先修复。

## What Changes

- **解除 8 人硬上限**:`lobby_ready_mask: u8` → 可扩展结构;`PlayerSlots::multi_player` 的 `assert!(count <= 8)` 移除
- **修正 simulation 层 2 人假设**:城市捕获从 0↔1 阵营互换改为"归 `last_attacker_faction`"(单机行为等价);`collect_command_players` 兜底 `if id <= 1` 移除
- **UI 参数化**:创建房间人数 2..=8 可 cycle;`current_players` 不再恒 1
- **完整掉线重连**:席位回收(`on_disconnect` 不永久剔除、重连复用 player_id);客户端接上 `apply_reconnect`,重建路径与正常网络路径**完全一致**(R1 强制),Disconnected 席位在 tick barrier 放行(R3),地图 `map_spec_hash` 一致性(R4)
- **分层修正**:世界重建逻辑移入 bevy_adapter,消除 render_view 直触仿真
- **合规补齐**:锁步回归测试(city_capture 多人 + 重连重放确定性)+ ADR × 2

无 BREAKING(单机 2 人行为等价;API 均向后兼容)。

## Capabilities

### New Capabilities
- `multiplayer-scale`: 玩家数量参数化 —— 创建房间人数 UI 可选 2..=8,`current_players` 正确上报,协议/配置不再假设 2 人

### Modified Capabilities
- `multiplayer-slots`: `PlayerSlots::multi_player` 解除 `assert!(count <= 8)` 上限
- `network-reconnect`: 完整重连链路 —— 席位回收、重连重建路径修正(init_simulation_world_multi + enable_ai:false)、客户端接通 `apply_reconnect`
- `relay-server`: tick barrier 对 Disconnected 席位放行(靠超时兜底);lobby ready 掩码可扩展
- `city-interaction`: 城市捕获归属改为 `last_attacker_faction`(替换 0↔1 互换)

## Impact

- **simulation**: `types.rs`(PlayerSlots)、`soldier/mod.rs`(city_capture_check_system)、`lib.rs`(collect_command_players、init_simulation_world_multi)
- **bevy_adapter**: `network.rs`(RelayServer:掩码/席位/重连)、`transport.rs`(客户端重连接线)、`driver.rs`(重连走 Network 管线)
- **render_view**: `lib.rs`(max_players 硬编码、地图硬编码、世界重建移入 bevy_adapter)、`ui/lan_lobby.rs`(人数 cycle)
- **测试**: network e2e 扩展、重连确定性测试、城市捕获多人测试、`cargo test` 全绿
- **文档**: ADR × 2(城市归属语义 + 重连恢复语义)、delta specs × 5

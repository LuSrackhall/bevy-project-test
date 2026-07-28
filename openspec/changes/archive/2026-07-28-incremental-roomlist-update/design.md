## Context

详见 `brainstorm-spec.md`。当前 `update_room_list` 每帧全量重建房间行，导致加入按钮的 `Pressed` 组件在 `Pointer<Press>`→`Pointer<Click>` 之间丢失。

## Goals / Non-Goals

**Goals:**
- 增量更新房间列表，保持按钮实体跨帧稳定
- 保持 `WidgetButton` + `On<Activate>` 模式

**Non-Goals:**
- 不改变 LAN 发现逻辑或网络层
- 不引入 BSN

## Decisions

### 增量更新策略

每帧执行：
1. 遍历已存在行（`Query<LanLobbyRowData>`），匹配 `RelayId`
2. `servers.servers` 按 `relay_id` 排序后处理
3. **移除**：`existing.relay_id` 不在 `servers` 中的行 → `despawn`
4. **新增**：`server.relay_id` 不在 existing 中的行 → `spawn`（含 `WidgetButton` + `On<Activate>`）
5. **更新**：存量行的 Text 内容（房间名、人数、状态）通过标记组件定位修改

### 标记组件

5 个轻量标记组件：
- `LanLobbyRowData(RelayId)` — 行级身份标签
- `RoomNameLabel`, `MapLabel`, `PlayersLabel`, `StateLabel` — Text 级定位标签

### 调度约束

`update_lan_servers` 在 `update_room_list` 之前注册，默认顺序正确。`.before()` 约束为可选加固，当前未显式添加（不阻塞功能）。

## Risks / Trade-offs

- [Risk] 文本更新需定位子实体 → 标记组件解决
- [Risk] 行顺序错乱 → 按 relay_id 排序解决
- [Risk] 时序竞争 → `.before()` 调度约束解决

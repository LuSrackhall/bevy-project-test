## Context

当前 LAN 房间列表由 `update_room_list` 系统每帧全量重建：despawn 所有行 + respawn。加入按钮（`WidgetButton` + `On<Activate>`）驻留在这些行中。

Bevy 0.19 的 `Pointer<Click>` 跨两帧执行（Press 帧 + Release 帧）。Press 设置 `Pressed` 组件，Release 检测 `Pressed` 后触发 `Activate`。但 `update_room_list` 在帧末 despawn 按钮实体，`Pressed` 随实体销毁丢失。Release 帧检测不到 `Pressed`，`Activate` 永不触发。

项目中所有其他按钮（"返回"、HUD 工具栏、菜单按钮等）均为一次性 spawn，实体跨帧稳定，故正常工作。仅加入按钮因每帧重建导致 `Pressed` 状态丢失。

## Goals / Non-Goals

**Goals:**
- 修复加入按钮点击无响应（`Activate` 不触发）
- 保持 `WidgetButton` + `On<Activate>` observer 模式，与项目其他按钮一致

**Non-Goals:**
- 不改变 LAN 发现协议
- 不改变网络传输层
- 不改变 BSN 方案（项目已排除 BSN）

## Decisions

### 方案：增量更新房间列表

将 `update_room_list` 从全量重建改为增量更新：

每次帧循环：
1. 对比已存在行（通过 `LanLobbyRowData` 中的 `RelayId` 匹配）与新发现的服务器列表
2. 移除消失的行
3. 添加新行（含 `WidgetButton` + `On<Activate>` observer 在按钮实体上）
4. 更新存量行的文本内容

**新增组件：**
- `LanLobbyRowData(RelayId)` — 行身份标签，用于匹配
- `RoomNameLabel`, `MapLabel`, `PlayersLabel`, `StateLabel` — Text 子实体标记组件，用于定位更新文本

**Stability:**
- 对 `servers.servers` 按 `relay_id` 排序保证行顺序稳定
- `update_lan_servers.before(update_room_list)` 确保数据源先更新

### 不选择方案的排除理由

| 方案 | 排除理由 |
|------|----------|
| Text 上继续调 | 问题不在 observer 位置。实体销毁后 Press→Click 事件链已断 |
| Interaction 旧模式 | Interaction 同是组件，随实体销毁 |
| 移出 spawn 到 setup | 大幅架构改动，回归风险高 |

## Risks / Trade-offs

- [Risk] 文本更新需标记组件 → [Mitigation] 新增 5 个轻量标记组件
- [Risk] 行顺序错乱 → [Mitigation] 按 relay_id 排序
- [Risk] Data 源时序 → [Mitigation] 加 `.before()` 调度约束

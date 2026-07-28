## ADDED Requirements

### Requirement: 增量更新房间列表

LAN 大厅的房间列表系统 `update_room_list` 必须使用增量更新，而非全量 despawn + respawn。

每帧执行增量操作：
1. 对比已存在行（`LanLobbyRowData.relay_id`）与服务清单
2. 移除消失的行
3. 添加新行（含 `WidgetButton` + `On<Activate>` observer 在按钮实体上）
4. 更新存量行的文本内容

#### Scenario: 新增房间行
- **WHEN** 新服务加入 `LanServers` 且其 `relay_id` 不在现有行中
- **THEN** 系统必须 spawn 新行（含加入按钮），按钮实体保持不变直到该行匹配的服务消失

#### Scenario: 移除消失行
- **WHEN** 已存在行的 `relay_id` 不在 `LanServers` 中（服务超时或关闭）
- **THEN** 系统必须 despawn 该行

#### Scenario: 保持实体稳定
- **WHEN** 服务持续存在（`relay_id` 连续出现在 `LanServers` 中）
- **THEN** 对应行实体保持不变（不被 despawn 和 respawn）

#### Scenario: 文本内容更新
- **WHEN** 存量行的服务信息变化（房间名、人数、状态）
- **THEN** 系统必须更新对应 Text 子实体的内容

#### Scenario: 点击加入按钮
- **WHEN** 加入按钮被点击（`Pointer<Press>` → `Pointer<Click>` 跨帧）
- **THEN** `Activate` 事件必须触发，`JoinRoomRequest.requested` 被设为 `true`

### Requirement: 行身份匹配

每行必须携带 `LanLobbyRowData(RelayId)` 组件用于身份匹配。

#### Scenario: 身份匹配
- **WHEN** 系统需要确定某行是否存在
- **THEN** 系统通过 `Query<LanLobbyRowData>` 对比 `RelayId` 而非依赖实体 ID 或位置

### Requirement: 数据源时序约束

`update_lan_servers` 必须在 `update_room_list` 之前执行。

#### Scenario: 调度顺序
- **WHEN** 两系统在同一帧运行
- **THEN** `update_lan_servers` 必须在 `update_room_list` 之前完成

### Requirement: 服务列表稳定排序

`servers.servers` 在 `update_room_list` 中按 `relay_id` 稳定排序后处理。

#### Scenario: 行顺序一致性
- **WHEN** 服务器列表变化（新增或移除）
- **THEN** 行显示顺序必须与排序后的一致

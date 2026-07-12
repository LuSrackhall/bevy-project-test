## ADDED Requirements

### Requirement: Room list rendering

LanLobby 页面从 `LanServers` resource 读取房间列表并动态渲染。

#### Scenario: Empty state

- **WHEN** `LanLobby` 进入且 `LanServers.servers` 为空
- **THEN** 显示空状态文案"没有找到局域网房间"

#### Scenario: Room appears in list

- **WHEN** `update_lan_servers` 将新房间加入 `LanServers`
- **THEN** 该房间的行立即出现在列表中，显示：房间名、地图、人数、状态

#### Scenario: Room disappears on TTL

- **WHEN** 房间超过 `LAN_TIMEOUT`（5 秒）未收到心跳
- **THEN** 该房间自动从列表中移除

#### Scenario: Playing rooms are disabled

- **WHEN** 房间的 `state` 为 `RoomState::Playing`
- **THEN** 该行的加入按钮禁用或不可点击

#### Scenario: Full rooms are disabled

- **WHEN** `current_players >= max_players`
- **THEN** 该行的加入按钮禁用

#### Scenario: Own room shows without Join button

- **WHEN** `SessionController.current_relay_id() == Some(adv.relay_id)`
- **THEN** 该行不显示"加入"按钮，显示标识（如无法交互状态）

### Requirement: CreateRoomModal

点击"创建房间"按钮弹出 Modal，配置后创建房间。

#### Scenario: Open modal

- **WHEN** 点击"创建房间"按钮
- **THEN** 在 LanLobby 上弹出 Modal，覆盖层显示

#### Scenario: Close modal via Cancel

- **WHEN** Modal 打开时点击"取消"
- **THEN** Modal 关闭，不创建房间

#### Scenario: Create room successfully

- **WHEN** 在 Modal 中填写配置并点击"创建房间"
- **THEN** 发射 `CreateRoomIntent`，Modal 关闭；集成系统消费 Intent 并调用 `SessionController.create_session()`

#### Scenario: Create room fails

- **WHEN** `SessionController.create_session()` 返回 `Err`
- **THEN** Modal 显示错误消息，不关闭

### Requirement: CreateRoomIntent integration

Integration System 桥接 UI（`CreateRoomIntent`）和 `SessionController`。

#### Scenario: Intent consumed

- **WHEN** `CreateRoomIntent` 被发射
- **THEN** `handle_create_room` system 读取 Intent 并调用 `SessionController.create_session(room)`

#### Scenario: Room not injected into LanServers

- **WHEN** 创建成功后
- **THEN** 不直接写入 `LanServers`，等待 LAN Discovery 自然更新

### Requirement: SessionController.current_relay_id

SessionController 新增只读查询方法，供 UI 判断"自己的房间"。

#### Scenario: Returns Some when active

- **WHEN** `SessionController` 有活跃 Session
- **THEN** `current_relay_id()` 返回 `Some(RelayId)`

#### Scenario: Returns None when inactive

- **WHEN** `SessionController` 没有活跃 Session
- **THEN** `current_relay_id()` 返回 `None`

## Context

LanLobby 已作为占位页面存在（#9），显示"正在扫描局域网房间..."和返回按钮。#5 提供了 `LanServers` / `RoomMetadata` 数据源，通过 `update_lan_servers` 系统驱动。#6 提供了 `SessionController` 生命周期管理。需要将三者串联为完整的房间列表 UI。

## Goals / Non-Goals

**Goals：**
- 单列房间列表，从 `LanServers` 动态渲染
- 每行显示：房间名、地图、人数、状态、操作按钮
- 空状态、TTL 消失、Playing/已满房间禁用
- CreateRoomModal：配置房间名 + 地图 + 人数
- `CreateRoomIntent` → Integration System → `SessionController`
- 自己的房间不显示"加入"按钮（通过 `Controller.current_relay_id()` 比对 `RoomAdvertisement.relay_id`）
- 创建成功后只关闭 Modal，不手动写入 LanServers（等待 LAN Discovery 自然更新）
- 新增 `SessionController.current_relay_id()` 查询方法

**Non-Goals：**
- 不处理加入逻辑（#8）
- 不实现房间等待页（#11）
- 不实现 "开始游戏" 按钮（#11）
- 不新增 GameState
- UI 不直接调用 SessionController

## Decisions

### AD1: 布局与交互

```
┌──────────────────────────────────────────────────┐
│                 局域网大厅                         │
├──────────────────────────────────────────────────┤
│ 房间名称        地图       人数      状态   操作  │
│ Alice 房       grassland  1/8      等待中 [加入] │
│ Bob 的局       desert     3/8      等待中   ⚡    │ ← 自己的房间
│ Test Room      islands    8/8      游戏中 (满员) │ ← 禁用
├──────────────────────────────────────────────────┤
│        没有找到局域网房间                          │ ← 空状态
├──────────────────────────────────────────────────┤
│ [返回]                           [创建房间]       │
└──────────────────────────────────────────────────┘
```

房间 TTL：5 秒无心跳自动移除（复用现有 `LAN_TIMEOUT`）。
自己的房间：`session_controller.current_relay_id() == Some(adv.relay_id)`。

### AD2: Intent 驱动的创建流程

```rust
// UI 层只发射 Intent
struct CreateRoomIntent {
    room_name: String,
    map_id: String,
    max_players: u8,
}

// Integration System（属于 #7，不在 UI 组件内）
fn handle_create_room(
    mut intents: EventReader<CreateRoomIntent>,
    mut controller: ResMut<SessionController>,
    mut commands: Commands,
) {
    for intent in intents.read() {
        let room = RoomMetadata {
            room_id: RoomId(/* 由 SessionController 生成 */),
            room_name: intent.room_name,
            map_id: intent.map_id,
            current_players: 1,
            max_players: intent.max_players,
            state: RoomState::Waiting,
        };
        match controller.create_session(room) {
            Ok(_) => { /* 关闭 Modal */ }
            Err(e) => { /* 显示错误 */ }
        }
    }
}
```

创建成功后不手动写入 `LanServers`。Beacon 链路：`SessionController 启动 Relay → Relay 广播 Beacon → LAN Discovery 接收 → LanServers 更新 → UI 自然出现自己的房间`。

### AD3: SessionController 新增查询方法

```rust
impl SessionController {
    /// 供 UI 判断"自己的房间"用。
    pub fn current_relay_id(&self) -> Option<RelayId> {
        self.session.as_ref().map(|s| s.relay.relay_id())
    }
}
```

### AD4: Modal 配置项

创建房间 Modal 包含：
- 房间名：文本输入（可选，默认自动生成）
- 地图：下拉选择 Small / Medium / Large / Huge
- 人数：下拉选择 2-8

使用 Bevy UI Widgets 的 Button + 计数循环（复用现有 `menu.rs` 的玩家数量选择器模式）。

## Risks / Trade-offs

- **[R1] 创建成功后自己的房间不会立即出现**：需等待 3 秒 Beacon 广播间隔。这是在维护 Beacon 权威性和即时反馈之间的取舍。当前接受 3 秒延迟。
- **[R2] current_relay_id 依赖 #6**：方法体在 #6 归档中，需要确认 `RelayHandle::relay_id()` 已在 main 上可用。
- **[R3] LanServers 在 LanLobby 退出时清除**：现有 `stop_lan_discovery` 已处理。

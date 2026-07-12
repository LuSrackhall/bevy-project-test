## Why

LanLobby 已有占位页面（#9）和 `LanServers` 发现数据源（#5），但列表仍是静态占位文本。需要将房间发现数据渲染为可交互的房间列表，并提供创建房间的入口，使局域网模式达到可用状态。

## What Changes

- 重写 `lan_lobby.rs`：单列房间列表，从 `LanServers` 动态渲染
- 新增 `CreateRoomIntent` Event + Integration System（消费 Intent 调用 `SessionController`）
- 新增 `SessionController::current_relay_id()` 查询方法（供 UI 判断"自己的房间"）
- 新增 `CreateRoomModal`：弹出层配置房间名/地图/人数
- 新增 `LanLobbyRoomRow` 组件：每行显示房间名、地图、人数、状态、操作按钮
- 房间行根据 `RoomState` 和人数显示不同的操作状态（可加入/自己的房间/已满/游戏中）

## Capabilities

### New Capabilities
- `room-list-ui`: 房间列表 UI + CreateRoomModal + CreateRoomIntent Integration System

### Modified Capabilities
<!-- 无现有 spec 变更 -->

## Impact

- `render_view/src/ui/lan_lobby.rs`：从占位页面重写为功能完整的房间列表
- `crates/bevy_adapter/src/session_host/controller.rs`：新增 `current_relay_id()` 方法（非破坏性）
- `render_view/src/ui/lan.rs`：`LanServerEntry` 和 `update_lan_servers` 已就绪，不需要改
- `render_view/src/ui/mod.rs`：可能需要注册新的 system（Integration System）
- 需要 `SessionController` 在 `render_view` 中作为 Bevy Resource 可用

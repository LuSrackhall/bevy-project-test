## Context

详见 `brainstorm-spec.md` 中的 AD1-AD4 设计决策。实现分三层：UI 层（lan_lobby.rs）、Resource 层（CreateRoomRequest）、Integration 层（handle_create_room system）。

## Goals / Non-Goals

**Goals:**
- 完整的房间列表 UI 替换占位页面
- CreateRoomModal + Intent 驱动的创建流程
- Integration System 桥接 UI 和 SessionController
- 自己的房间判断逻辑（current_relay_id 比对）

**Non-Goals:**
- 不实现加入逻辑（#8）
- 不实现房间等待页（#11）
- 不实现地图预览

## Decisions

### 数据流

```
UDP Beacon → LanDiscoveryListener → LanServers resource
                                          ↓
lan_lobby.rs (动态渲染列表) ← update_lan_servers system
     ↓
[创建房间按钮] → CreateRoomRequest Resource (requested=true)
                       ↓
handle_create_room system (Integration Layer)
                       ↓
          SessionController::create_session()
                       ↓
      Relay 启动 → Beacon 广播 → LanServers 自然更新
```

### 模块变更

```
render_view/src/ui/lan_lobby.rs  — 重写：列表 + Modal
render_view/src/lib.rs            — 注册 handle_create_room system
bevy_adapter::session_host::controller.rs — 新增 current_relay_id()
```

### SessionController 的 Bevy Resource 注入

```rust
// 在 render_view 或 bevy_adapter 插件中
app.insert_resource(SessionController::new(Box::new(ThreadRelayRuntime)));
```

由于 `ThreadRelayRuntime` 在 `bevy_adapter::session_host` 中，而 UI 在 `render_view`，注入点需要在 `bevy_adapter::BevyAdapterPlugin` 或应用根插件中。

### 测试策略

- UI 测试：通过 Bevy 的 App 测试框架验证 `LanServers` → 房间行渲染
- Integration 测试：验证 `CreateRoomRequest` → `SessionController.create_session` 被调用
- 自己的房间逻辑：验证 `current_relay_id` 匹配/不匹配时的 UI 状态

## Risks / Trade-offs

- **[R1] 创建后房间 3 秒后才出现**：Beacon 广播间隔。已在 brainstorm-spec 中接受。
- **[R2] SessionController 注入时机**：需确保在 UI 系统之前初始化。

## 1. SessionController 新增 current_relay_id

- [x] 1.1 在 `controller.rs` 的 `impl SessionController` 中添加 `pub fn current_relay_id(&self) -> Option<RelayId>`
- [x] 1.2 编译检查确认不破坏现有 API

## 2. CreateRoomRequest Resource + Integration System

- [x] 2.1 在 `render_view/src/lib.rs` 中定义 `CreateRoomRequest` Resource（包含 `requested`、`room_name`、`map_id`、`max_players`）
- [x] 2.2 实现 `handle_create_room` Integration System：读取 `CreateRoomRequest` → 构建 `RoomMetadata` → 调用 `SessionController::create_session()`
- [x] 2.3 在 `render_view` 插件中注册 `CreateRoomRequest` Resource 和 `handle_create_room` System
- [x] 2.4 SessionController 作为 Bevy Resource 注入（在 BevyAdapterPlugin 中），添加 `Send + Sync` bounds 到 traits

## 3. 房间列表 UI

- [x] 3.1 重写 `lan_lobby.rs::setup_lan_lobby`：构建顶层容器 + 列标题行 + 空状态文案
- [x] 3.2 实现房间行渲染组件 `LanLobbyRow`，从 `LanServers` 读取数据
- [x] 3.3 实现 `update_room_list` system：每帧读取 `LanServers` 并动态更新房间行
- [x] 3.4 实现加入按钮状态：Playing/已满 → 禁用，自己的房间 → 不显示加入按钮
- [x] 3.5 实现空状态显示/隐藏切换
- [x] 3.6 点击房间行判断逻辑（能否加入、是否自己的房间）

## 4. CreateRoomModal

- [x] 4.1 实现 Modal 容器 UI（覆盖层 + 居中面板）
- [x] 4.2 实现房间名输入/默认字段
- [x] 4.3 实现地图尺寸选择器（复用 `MapSizeBtn` 模式）
- [x] 4.4 实现人数选择器（2-8）
- [x] 4.5 实现"创建房间"按钮 → 设置 `CreateRoomRequest`
- [x] 4.6 实现"取消"按钮 → 关闭 Modal
- [x] 4.7 实现创建失败时的错误显示（错误日志输出）

## 5. 清理

- [x] 5.1 删除占位文本，`LanLobbyLayout` 组件已被替换为新的组件体系

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

## Tasks

### Step 1: 非阻塞连接（`LobbyConnectionStatus`）

**File:** `crates/bevy_adapter/src/transport.rs`

- [ ] 新增 `LobbyConnectionStatus` 结构（`Arc<Mutex<Option<Result<(), String>>>>`）
- [ ] 新增 `spawn_network_client_async` 函数，返回后不阻塞
- [ ] 原有 `spawn_network_client` 保持不变

### Step 2: Lobby UI（`lobby.rs`）

**File:** `crates/render_view/src/ui/lobby.rs`

- [ ] 定义 `LobbyUI` marker component
- [ ] `setup_lobby_ui` 系统（OnEnter）：渲染三阶段 UI（Connecting / Connected / Failed）
- [ ] `cleanup_lobby_ui` 系统（OnExit）：销毁 UI
- [ ] 取消按钮 → 清理网络资源 → 返回 MainMenu

### Step 3: Lobby 状态机重写

**File:** `crates/render_view/src/lib.rs`

- [ ] `setup_lobby_system` 改为非阻塞：发起 TCP 连接 + 注册 `LobbyConnectionStatus`
- [ ] 新增 `lobby_update_system`：轮询 `LobbyConnectionStatus`，驱动 UI 阶段
- [ ] 当 GameStarted 收到后转入 Playing

### Step 4: 主菜单修复

**File:** `crates/render_view/src/ui/menu.rs`

- [ ] "开始联机"按钮 target 改为 `GameState::Lobby`
- [ ] UI 简化：移除笨拙的文本输入，改为简洁入口

### Step 5: 注册系统

**File:** `crates/render_view/src/ui/mod.rs`

- [ ] 注册 `OnEnter(Lobby)` / `OnExit(Lobby)` / `Update(Lobby)` 系统

### Step 6: 测试

- [ ] 模拟 Lobby → Playing 转换
- [ ] 验证 Lobby UI 各阶段渲染
- [ ] 全部现有测试通过（114+22+1）

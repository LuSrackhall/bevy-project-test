## Why

`GameState::Lobby` 已存在但子菜单的"开始联机"按钮跳过它直达 `Playing`。Lobby 状态仅有 TCP 连接和 GameStarted 等待逻辑，无任何 UI 渲染（白屏）。TCP 连接阻塞主线程 30 秒，导致即使进入 Lobby 也无法渲染界面。玩家在"开始联机"后看到的只是冻结的白屏，没有任何视觉反馈。

## What Changes

- **FIX**: 主菜单"联机"按钮指向 `GameState::Lobby` 而非 `GameState::Playing`
- **NEW**: `crates/render_view/src/ui/lobby.rs` — Lobby 等待室 UI
- **NEW**: `LobbyState` 资源 + TCP 连接异步轮询（不阻塞 Bevy 主线程）
- **FIX**: `setup_lobby_system` 拆为触发+轮询两阶段
- 主菜单联机区域界面优化（移除笨拙的文本输入）

## Capabilities

### New Capabilities
- `lobby-waiting-room`: 联机大厅等待室 UI（连接状态/等待其他玩家/取消按钮）
- `lobby-async-connect`: TCP 连接异步轮询（不阻塞主线程）

### Modified Capabilities

（无现有 spec 被修改）

## Impact

| 系统 | 影响 |
|------|------|
| `crates/render_view/src/ui/lobby.rs` | 新增文件：Lobby UI 组件（~150-200 行） |
| `crates/render_view/src/ui/menu.rs` | 联机按钮指向 Lobby；简化联机区域输入 |
| `crates/render_view/src/ui/mod.rs` | 注册 OnEnter/OnExit/Update 系统 |
| `crates/render_view/src/lib.rs` | `setup_lobby_system` 非阻塞重构；新增 `LobbyConnectionState` |
| `crates/bevy_adapter/src/transport.rs` | 新增非阻塞连接变体或暴露 `connected_rx` |

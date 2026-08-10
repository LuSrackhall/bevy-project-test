## 1. 修复 lobby_update_system 同批丢 GameStarted

- [x] 1.1 Connected 阶段移除就绪 `LobbyUpdate` 分支后的 `return`(置 Ready 后继续迭代同批)
- [x] 1.2 同批仅含 LobbyUpdate 时保持 Ready 语义,后续批次 GameStarted 仍能开局

## 2. 回归测试(状态机级)

- [x] 2.1 `render_view/tests/lobby_system.rs`:同批 `[LobbyUpdate(本机就绪), GameStarted]` → 断言 `NetworkGameStart.received == true`
- [x] 2.2 分批场景:先 `[LobbyUpdate]` 进 Ready,再分帧 `[GameStarted]` → 仍开局

## 3. transport 级 e2e 锚点

- [x] 3.1 `bevy_adapter/tests/lobby_start_e2e.rs`:自动开局路径(双客户端上行 tick-1 帧)送达 GameStarted
- [x] 3.2 就绪开局路径(双客户端 LobbyReady)送达 GameStarted

## 4. 验证

- [x] 4.1 `cargo test --workspace` 全绿

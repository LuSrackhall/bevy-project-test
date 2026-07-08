## 1. 扩展协议类型

- [ ] 1.1 添加 `LobbyPlayerState` 结构体
- [ ] 1.2 RelayClientMessage 新增 `LobbyReady` 变体
- [ ] 1.3 RelayServerMessage 新增 `LobbyUpdate` 变体
- [ ] 1.4 PlayerTickFrame 新增 `version: u16` 字段

## 2. 实现 Relay 端 lobby 逻辑

- [ ] 2.1 RelayServer 新增 `lobby_ready_mask` + `on_lobby_ready()` 方法
- [ ] 2.2 relay/src/lib.rs 添加 LobbyReady match arm → 调用 on_lobby_ready → 广播 LobbyUpdate
- [ ] 2.3 transport.rs 添加 LobbyUpdate match arm（_ => {}）

## 3. 集成测试

- [ ] 3.1 在 `relay/tests/` 添加 lobby 协议测试（2 客户端 → ready → GameStarted）

## 4. 编译验证

- [ ] 4.1 运行 `cargo check -p bevy_adapter -p relay` 确认编译通过
- [ ] 4.2 运行 `cargo test -p relay` 确认全部通过

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run myspec-verify
2. **User Acceptance**: Present change summary
3. **Merge**: After user accepts, merge to main
4. **Archive**: openspec archive on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

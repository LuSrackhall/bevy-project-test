## 1. 通道扩展

- [ ] 1.1 NetworkEvent 新增 LobbyUpdate 变体
- [ ] 1.2 NetworkSender 新增 lobby_ready + send_lobby_ready()

## 2. transport.rs 适配

- [ ] 2.1 run_session() LobbyUpdate arm → push 到 event_receiver
- [ ] 2.2 write task 添加 lobby_ready 检查发送逻辑

## 3. Lobby 状态机 + 轮询

- [ ] 3.1 LobbyPhase 新增 Ready 阶段
- [ ] 3.2 lobby_update_system 读取 NetworkEvent::LobbyUpdate
- [ ] 3.3 收到 GameStarted → 转 Playing

## 4. UI 层

- [ ] 4.1 添加 Ready 按钮 + observer 调用 send_lobby_ready
- [ ] 4.2 添加玩家就绪状态文本

## 5. 编译验证

- [ ] 5.1 运行 cargo check 确认编译通过
- [ ] 5.2 运行 cargo test 确认全部通过

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run myspec-verify
2. **User Acceptance**: Present change summary
3. **Merge**: After user accepts, merge to main
4. **Archive**: openspec archive on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

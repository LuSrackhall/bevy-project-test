## 1. 协议和 relay

- [x] 1.1 LanDiscoveryPacket 结构体（network.rs）
- [x] 1.2 relay UDP 广播（relay/src/lib.rs）

## 2. 客户端监听

- [x] 2.1 LanDiscoveryListener Resource + tokio 线程
- [x] 2.2 bevy_adapter/src/lib.rs pub mod lan
- [x] 2.3 render_view LanDiscoveryServers + 系统

## 3. UI

- [ ] 3.1 主菜单 LAN 服务器列表
- [ ] 3.2 列表项点击直接连接

## 4. 编译验证

- [ ] 4.1 cargo check
- [ ] 4.2 cargo test -p simulation

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run myspec-verify
2. **User Acceptance**: Present change summary
3. **Merge**: After user accepts, merge to main
4. **Archive**: openspec archive on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

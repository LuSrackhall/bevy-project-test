## 1. 实现 UDP Beacon 广播

- [x] 1.1 修改 `run_local_relay` 签名，增加 `relay_id: RelayId` 和 `room: &RoomMetadata` 参数，移除 `seed` 和 `max_players`
- [x] 1.2 在 TCP 绑定 + port_tx.send 后，创建 UdpSocket 绑定 `0.0.0.0:{actual_port}` 并启用 `set_broadcast(true)`
- [x] 1.3 在主循环中使用 `tokio::select!` 集成 beacon 间隔（3 秒），广播 `LanDiscoveryPacket` 到 `255.255.255.255:9876` 和 `127.0.0.1:9876`
- [x] 1.4 所有 UDP 错误仅 log，不 panic

## 2. 适配调用侧

- [x] 2.1 `ThreadRelayRuntime::start()` 中将 `stop` 从 `&AtomicBool` 改为 `Arc<AtomicBool>`
- [x] 2.2 `start()` 中生成随机 `RelayId`，同时传给 `run_local_relay` 和 `ThreadRelayHandle`

## 3. 验证

- [x] 3.1 编译通过（`cargo build -p bevy_adapter`）
- [x] 3.2 运行 `cargo run -- --windowed`，创建房间，确认自己房间出现在列表中
- [x] 3.3 现有测试通过（`cargo test -p bevy_adapter`）

---

## Post-Implementation Workflow

<!-- DO NOT MODIFY THIS SECTION — it defines the required workflow after all tasks are complete -->

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/<name>`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

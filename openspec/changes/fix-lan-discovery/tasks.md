## 1. 测试先行(宪法 §10.1)

- [ ] 1.1 packet 单测(R3):LanDiscoveryPacket encode/decode round-trip + 错误 magic/garbage → decode None
- [ ] 1.2 `LanDiscoveryListener::start_on(port)` API(默认 9876,测试隔离用)(D3)
- [ ] 1.3 绑定冲突测试(S1):listener 持 0.0.0.0:9876 时裸绑 0.0.0.0:9876 → EADDRINUSE(证明旧路径必冲突)
- [ ] 1.4 **生产路径 e2e(R1)**:LanDiscoveryListener(9876)运行 → ThreadRelayRuntime::start → 轮询 drain(≤5s)断言收到包且 relay_id 匹配;预修必挂
- [ ] 1.5 源端口探针(R2):独立 socket 断言 beacon 源端口 ≠ 9876
- [ ] 1.6 去重语义(S2):同 relay_id 更新、异 relay_id 新增

## 2. 修复

- [ ] 2.1 thread.rs beacon bind `0.0.0.0:0`(原 0.0.0.0:9876)+ set_broadcast(true)

## 3. 收尾

- [ ] 3.1 回归:全仓 `cargo test` + `cargo check` 全绿;宪法自检清单逐项过
- [ ] 3.2 防火墙文档提示(入站 UDP 9876 + relay 临时端口)

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

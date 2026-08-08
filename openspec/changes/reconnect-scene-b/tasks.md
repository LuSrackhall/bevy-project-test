## 1. 测试先行(状态机级单测,宪法 §10.1)

- [x] 1.1 handle_reconnect(last=0) 返回全量日志 + map_size 字段单测
- [x] 1.2 relay 补发 GameStarted 单测:复用 Disconnected 席位且 game_started → GameJoined 后补发;已 Playing 无害
- [x] 1.3 A/B 判定边界单测:Playing 断在 tick0 → Scene A 不重建;全新进程 → Scene B
- [x] 1.4 防重复重建单测:Playing 期间二次重连响应 → Scene A apply,不重建

## 2. 协议变更

- [x] 2.1 `ReconnectResponse` 增 `map_size: MapSize`;handle_reconnect 携带

## 3. relay 端补发 GameStarted

- [x] 3.1 on_join_game 复用 Disconnected 席位时返回「重连」信号
- [x] 3.2 relay_core JoinGame 分支:game_started 且重连 → GameJoined 后 broadcast GameStarted

## 4. 客户端 transport 重试 + 门控

- [x] 4.1 无条件 ReconnectRequest(去掉 last>0 门控)
- [x] 4.2 JoinRejected/Error 有界重试(上限 10 次,指数退避 1s→30s),不再按 last==0 放弃
- [x] 4.3 回放期上发门控:network_flush_system 在 catch-up 期间跳过

## 5. driver 快速回放 + render_view 过渡

- [x] 5.1 driver 增 catch_up 状态 + 批量 run_tick 追赶模式(复用 handle_seek 批量逻辑)
- [x] 5.2 采用更简洁方案:GameStarted 增 map_size,lobby 从 GameStarted 取 seed+map_size 转 Playing(ReconnectResponse 分支非必需)
- [x] 5.3 reset_game_system 网络模式用 NetworkGameStart.map_size(替代硬编码 Medium)
- [x] 5.4 session/reconnect.rs `rebuild_world` 增 map_size 参数

## 6. 测试扩展 + 集成

- [x] 6.1 进程重启重建+快速回放 hash 全等(MoveTo 真实命令 + 确定性逐 tick 步进;Phase A 在线 vs Phase B 重建两相位比对)
- [x] 6.2 map_size 一致性:非默认 MapSize 重建与在线客户端 hash 全等
- [x] 6.3 席位复用集成测试(test_relay_resends_game_started:断开→清扫→新进程复用席位);客户端重试(return false)由代码变更覆盖
- [x] 6.4 Scene A 回归 + 全仓 `cargo test`/`cargo check` 全绿;宪法自检清单逐项过

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

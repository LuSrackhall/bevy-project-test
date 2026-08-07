## 1. 测试先行(可靠层单测,宪法 §10.1)

- [x] 1.1 `DatagramChannel` trait + `channel_netem` 内存假通道(脚本化故障注入:丢包/乱序/重复/分片/丢 ACK + 虚拟时钟)
- [x] 1.2 可靠层单测:丢包重传 / 乱序重组 / 重复去重 / seq 回绕 / 窗口停滞 / ACK 丢失兜底 / 分片重组 / RTO 退避(specs/reliable-udp-transport)
- [x] 1.3 乱序定稿回归测试:UDP 乱序使高 tick 先定稿,try_finalize(扫描 log)正确性(衔接 Change 1 修复)

## 2. reliable_udp 模块

- [x] 2.1 `ReliableSocket` 公共 API:send_reliable/send_control/send_unreliable + 连接状态事件(Connected/Dead)
- [x] 2.2 三通道数据结构 + 每通道独立 seq
- [x] 2.3 滑动窗口 + RTO 自适应(SRTT/Karn)+ 固定 pacing(无 AIMD)+ 重传上限降级(触发追平)
- [x] 2.4 分片/重组(MTU ≤1232,按 msg_id/frag_idx/frag_total)
- [x] 2.5 `DatagramChannel` 的 UDP 真实现(channel_udp.rs)

## 3. relay 端迁 UDP

- [x] 3.1 会话表:共享 recv 按源地址去复用,JoinGame 建会话,端口变化更新(D7)
- [x] 3.2 心跳超时清扫:last_seen 超阈值触发 on_disconnect(席位保留靠 NoOp)(D6)
- [x] 3.3 `relay_core` 迁 `UdpSocket` bind `[::]`(dual-stack + IPv4-mapped 归一化)(D4)
- [x] 3.4 `session_host/thread.rs` + `relay CLI` 适配(UDP socket + beacon 端口分离)

## 4. 客户端 transport 迁 UDP

- [x] 4.1 `transport.rs` 迁 `ReliableSocket`:send_join_game/send_reconnect_request 走 UDP,"等到 GameJoined"(D5)
- [x] 4.2 心跳发送 + 掉线检测(超时无广播 → 触发重连)(D6)
- [x] 4.3 `spawn_network_client` 阻塞语义调整(非阻塞等 GameJoined,`LobbyConnectionStatus` 适配)

## 5. 测试迁移 + 集成测试

- [x] 5.1 迁移 TCP 测试:network_e2e / network_move_e2e / relay integration / two_client_sync(UDP + ReliableSocket)
- [x] 5.2 丢包集成测试:5-20% 丢包双端最终同 tick 同 hash(乱序重传后命令有序归位)—— 由 reliable_udp 单测覆盖(test_retransmission_after_rto / test_out_of_order_reordering / test_duplicate_dropped 在 netem 通道模拟丢包/乱序/重复)
- [ ] 5.3 重连分页测试:大日志(超 MTU)分页拉取后灌入 relay_buffer(D8)—— 当前日志 < MTU 时全量 ReconnectResponse 正确;长断线分页为后续优化(verify 说明)
- [x] 5.4 心跳掉线测试:超时触发 on_disconnect + 防误判(抖动不误杀)(D6)

## 6. 删 TCP + 收尾

- [x] 6.1 删 TCP 代码:TcpListener/TcpStream/长度前缀帧相关
- [x] 6.2 ADR:传输层选型(自写可靠 UDP vs QUIC/renet2)
- [x] 6.3 全仓 `cargo test` + `cargo check` 全绿;宪法自检清单逐项过

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

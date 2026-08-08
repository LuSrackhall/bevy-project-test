## 1. 测试先行(状态机级单测,宪法 §10.1)

- [x] 1.1 分页边界单测:log 长 65 tick,last=0 → total_ticks=65、page_count=3、页=32/32/1;页内 tick 升序、页间首 tick 连续、first_tick=last+1
- [x] 1.2 **乱序定稿分页单测**:log append 序乱序,分桶后各页覆盖完整无缺口(CRITICAL #1 回归)
- [x] 1.3 边界:`total_ticks=0`(page_count=0)、`page_count=1` 退化、`frozen` → Error
- [x] 1.4 客户端渐进应用单测:元数据 + 顺序页 → relay_buffer 连续覆盖 `[first..=first+total-1]`;应用页 0 后页 0 tick ready、页 1 未 ready
- [x] 1.5 防御校验单测:乱序页拒绝、重复页拒绝、越界拒绝、页头 page_count 与元数据不符拒绝、`tick <= last_tick_consumed` 拒绝
- [x] 1.6 限定 clear 单测:buffer 含陈旧项(`< first`)与实时项(`> first+total`),apply 元数据后陈旧删、实时留(CRITICAL #2 回归)
- [x] 1.7 续传单测:二次 handle_reconnect(last 推进后)返回 first 无重叠无缺口

## 2. 协议变更

- [x] 2.1 `ReconnectResponse` 元数据化(移除 ticks,增 total_ticks/page_count)+ `ReconnectPage` 结构体(network.rs)
- [x] 2.2 `RelayServerMessage` 增 `ReconnectPage` 变体;`NetworkEvent` 增 `ReconnectPage` 变体

## 3. relay 端分页

- [x] 3.1 `handle_reconnect` 改分桶:按 tick 值区间 `[first+i*PAGE_TICKS, first+(i+1)*PAGE_TICKS)` 从 log 筛条目(PAGE_TICKS=32,const);total_ticks=计数;frozen → Err(D2/D5)
- [x] 3.2 relay_core Reconnect 分支:回元数据后循环推 page_count 页到 Control(send_reliable(1, ...))(D1/D6)

## 4. 客户端渐进应用

- [x] 4.1 `NetworkCommandSource` 增页游标(reconnect_meta + reconnect_next_page)
- [x] 4.2 `apply_reconnect` 拆分:元数据校验 + `relay_buffer.retain(>= first)` 限定 clear + 设置游标
- [x] 4.3 新增 `apply_reconnect_page`:page_count/游标校验 + 插入 ticks + 推进游标
- [x] 4.4 `udp_session` 分发 ReconnectPage 事件;transport.rs `resp.ticks.len()` 引用改 total_ticks
- [x] 4.5 `reconnect_recovery_system` 显式排在 `network_poll_system` 之前(bevy ordering)(D3)

## 5. 测试扩展 + 集成

- [x] 5.1 reconnect_catchup.rs 多页化:单响应 helper 换元数据+多页 helper,断言追平至同 tick
- [x] 5.2 多页重连集成测试(relay/tests/integration.rs):真实 UDP,33 tick → 2 页,元数据 + 页按 Control 序到达且覆盖完整;确定性分页逻辑由单测覆盖(替代假通道 hash 构造,更贴近真实链路)
- [x] 5.3 真实 UDP 冒烟:5.2 即 >单页边界的冒烟;>MTU 页分片由 reliable_udp 分片单测覆盖(非 CI 门禁)
- [x] 5.4 回归:全仓 `cargo test` 198 全绿 + `cargo check` 通过;宪法自检清单逐项过

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

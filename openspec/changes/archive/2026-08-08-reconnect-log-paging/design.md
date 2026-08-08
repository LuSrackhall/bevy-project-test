## Context

Change 2 交付可靠 UDP 后,重连恢复仍走**单条全量 ReconnectResponse**(network.rs:647 `handle_reconnect` 返回 `ticks: Vec<TickCommands>` 完整日志;network.rs:286 `apply_reconnect` 一次灌入 relay_buffer)。可靠层透明 MTU 分片保证功能正确,但长断线(数百 KB)时客户端须重组完整消息才交付首 tick,无渐进性。本文落地 brainstorm-spec 的 D1-D7 分页设计,深入实现架构、数据流、错误处理与测试策略。宪法约束:分层(§1)、确定性(§0.1)、lockstep 六步(§3)、NoOp 兜底(§3.2)。

## Goals / Non-Goals

**Goals:**
- 元数据 + ReconnectPage 推送分页,客户端**边收边放**
- 按 tick 值分桶(日志 append 序非 tick 序),页覆盖完整无缺口
- 限定范围 clear,不吞页面范围外实时 broadcast(确定性)
- 页游标 + 一致性校验,跨会话 stale 页防御
- 全部测试确定性(状态机单测零 sleep;长断线用假通道+注入时钟)

**Non-Goals:**
- 加密 / wasm / 权威服务器;可靠层分片改动;日志压缩
- relay 内存严格有界(可靠传输须持有未 ACK 帧,内存 O(日志子集)不可避免)
- Scene B(进程重启世界重建)的 seed/players 真正使用(既有缺口,不在本 change)

## Decisions

### 数据结构

```rust
// network.rs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconnectResponse {   // 元数据化(移除 ticks)
    pub game_id: u64,
    pub ruleset_version: u32,
    pub seed: u64,
    pub map_spec_hash: u64,
    pub first_tick: u32,          // = last_tick_consumed + 1
    pub total_ticks: u32,         // log 中 tick > last_tick_consumed 的计数
    pub page_count: u32,          // ceil(total_ticks / PAGE_TICKS),0 当 total_ticks=0
    pub players: Vec<PlayerState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReconnectPage {
    pub page_index: u32,          // 0..page_count
    pub page_count: u32,          // 与元数据一致,防 stale
    pub first_tick: u32,          // 本页起始 tick(用于防御校验)
    pub ticks: Vec<TickCommands>,
}
// RelayServerMessage 增 ReconnectPage 变体;NetworkEvent 增 ReconnectPage 变体
```

### relay 侧数据流(relay_core.rs + network.rs)

1. `RelayServer::handle_reconnect(&self, req)` 只构建元数据:
   - `first_tick = req.last_tick_consumed + 1`
   - `total_ticks = self.log.iter().filter(|b| b.tick > req.last_tick_consumed).count()`
   - `page_count = total_ticks.div_ceil(PAGE_TICKS)`
   - `frozen` 时返回 `Err`(Error 消息)
2. relay_core `handle_message` Reconnect 分支:先 `send_reliable(1, 元数据)`,然后循环 `i in 0..page_count` 构建第 i 页并 `send_reliable(1, ReconnectPage)`。
3. **分桶(D2)**:第 i 页覆盖 `[first + i*PAGE_TICKS, first + (i+1)*PAGE_TICKS)`;**从 log 中筛出 tick 落在该区间的条目按值归入**(log 是 append 序,不能按位置切)。`PAGE_TICKS = 32`。
4. **推页管道**:页面在 sender unacked 队列按 window(32 帧)+ pacing 1ms 排空;客户端 Control 通道 ReliableOrdered 保证逐页顺序。内存 O(日志子集)是可靠传输固有,不追求严格有界。
5. **页面与 broadcast 不重叠**:页面止于请求时刻最后定稿 tick;之后新定稿走实时 broadcast。日志 append-only,快照边界一致。

### 客户端数据流(transport.rs + network.rs)

1. `udp_session`:`ReconnectResponse`(元数据)→ `NetworkEvent::Reconnect(resp)`;`ReconnectPage` → `NetworkEvent::ReconnectPage(page)`。`resp.ticks.len()` 日志引用改 `resp.total_ticks`。
2. `reconnect_recovery_system` 处理两类事件,**显式排在 `network_poll_system` 之前**(D3,避免元数据应用晚于实时 broadcast 入 buffer 被 clear)。
3. `NetworkCommandSource` 增页游标字段:
   ```rust
   reconnect_meta: Option<(u32 first_tick, u32 total_ticks, u32 page_count)>,
   reconnect_next_page: u32,
   ```
4. `apply_reconnect(&meta, expected)`:
   - ruleset_version 校验(mismatch → Err)
   - `relay_buffer.retain(|tick, _| *tick >= meta.first_tick)`(D3 限定 clear,只删陈旧)
   - 设置 `reconnect_meta`、`reconnect_next_page = 0`、`connected = true`
5. `apply_reconnect_page(&page, expected)`:
   - 校验 `page.page_count == meta.page_count`(stale 防御)与 `page.page_index == reconnect_next_page`(乱序/重复拒绝)
   - 拒绝 `tick <= last_tick_consumed`
   - 逐条 `relay_buffer.insert(batch.tick, batch.clone())`
   - `reconnect_next_page += 1`;当 `== page_count` 时重放完成(relay_buffer 已含全部缺口 tick)
6. **driver 追平**:driver.rs:313 `is_tick_ready(next_tick)` false 时停步;页面渐进灌入后自然追平多 tick。`last_tick_consumed` 由 network_flush_system 每次 flush 推进(transport.rs:143),重连再掉线自动从已应用点续传。

### 边界与错误处理

- `total_ticks = 0`:`page_count = 0`,客户端收元数据后立即重放完成(不等待页)。
- `page_count = 1`:退化单页,等价旧路径。
- 元数据 ruleset_version mismatch → Error,客户端重试(重连语义与 Change 2 一致:last_tick_consumed>0 → 重试)。
- 非法页(乱序/重复/越界/页头不符):丢弃并日志,不崩溃;若可靠层真丢页(理论上不发生),页游标停滞 → 客户端 3s 掉线检测 → 重新重连兜底。

### 测试策略

**单测(network.rs 状态机级,零 sleep):**
- 分页边界:log 长 65 tick,last=0,PAGE_TICKS=32 → total_ticks=65、page_count=3、页=32/32/1;页内 tick 升序、页间首 tick 连续、`first_tick=last+1`
- **乱序定稿分页**:log append 序乱序,断言分桶后各页覆盖完整无缺口(CRITICAL #1 回归)
- `total_ticks=0`、`page_count=1` 退化
- 客户端:元数据 + 顺序页 → relay_buffer 连续覆盖 `[first..=first+total-1]`;渐进断言(应用页 0 后页 0 tick ready、页 1 未 ready)
- 防御:乱序页拒绝、重复页拒绝、越界拒绝、页头 page_count 与元数据不符拒绝
- 限定 clear:buffer 含 `tick < first` 陈旧项 + `> first+total` 实时项,apply 后陈旧删、实时留
- 续传:二次 handle_reconnect(last 推进后)返回 first 无重叠无缺口

**集成:**
- reconnect_catchup.rs 多页化:把单响应 helper 换成元数据 + 多页 helper,复用现有 App/SimulationDriver 脚手架,断言追平至同 tick
- 确定性长断线回放:relay_core 假 DatagramChannel + 注入时钟(替代真实 UDP + sleep),断线 100 tick、3 页重连,追平后与未断线对照 ReplayRecorder hash 全等
- 保留一个真实 UDP >MTU 冒烟(非 CI 门禁)

## Risks / Trade-offs

- [日志乱序 → 页内缺口 driver 死等] → D2 按 tick 值分桶 + 乱序定稿分页单测回归
- [元数据应用吞实时 broadcast → 确定性缺口] → D3 限定 clear(retain ≥ first)+ reconnect_recovery_system 排在 network_poll_system 前
- [跨会话 stale 页被误收] → D4 页游标 + page_count 一致性校验;新元数据重置游标
- [推模型与已批准 D8 拉模型分歧] → brainstorm 显式记录偏离理由(§16.3)
- [长断线 e2e flaky] → 假通道 + 注入时钟确定性构造;真实 UDP 冒烟不门禁
- [relay 内存非严格有界] → 明确取舍(可靠传输须持有未 ACK 帧);收益是客户端渐进交付

## Migration Plan

1. 测试先行:network.rs 分页/序/边界单测(零 sleep)
2. 协议:ReconnectResponse 元数据化 + ReconnectPage 变体(RelayServerMessage/NetworkEvent)
3. relay:handle_reconnect 分桶 + relay_core 推页
4. 客户端:页游标 + apply_reconnect 拆分 + 系统顺序固定
5. 测试扩展:reconnect_catchup 多页化 + 确定性长断线回放 + 续传/竞态单测
6. 回滚:分支级 revert(协议不兼容,两端同仓)

## Open Questions

- PAGE_TICKS 最优值(32 初值,分片数/进度粒度权衡)
- 分批推页是否需要 in-flight gating(现按 sender window+pacing 排空;若 profile 显示峰值,后续加)
- 页内防御性去重是否需要(可靠层已帧级去重,页面级覆盖完整性由分桶保证)

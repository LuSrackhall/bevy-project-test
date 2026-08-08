## Context

Change 2 已交付自写可靠 UDP。当前重连恢复:客户端发 `ReconnectRequest`,relay `handle_reconnect`(network.rs:647)返回**全量** `ReconnectResponse.ticks`(last_tick_consumed+1 至当前定稿 tick 的完整命令日志),经 Control 可靠通道 + 透明 MTU 分片传输(frag_total u16,~78MB 上限,无正确性瓶颈),客户端 `apply_reconnect`(network.rs:286)一次性灌入 relay_buffer。

**已知限制(Change 2 verify 记录)**:长断线(数分钟 → 几百 KB)时单消息方案:(1) relay 一次性构建整个 blob;(2) 客户端必须重组完整消息后才交付任意 tick(无渐进);(3) 无逐页进度。**功能正确,但无渐进回放**。

经 3 子 agent 多维度审查(代码架构/宪法合规/测试策略),设计整合全部关键发现:
- CRITICAL:日志**非按 tick 有序**(append 顺序,有 out-of-order finalize 测试佐证)——分页必须按 tick 值分桶,不能按 log 位置切片,否则页内缺口导致 driver 死等。
- CRITICAL:元数据应用时 `relay_buffer.clear()` 会吞掉页面范围外的实时 broadcast——须限定范围 clear。
- 宪法:核心合规;推模型偏离已批准 D8 拉模型须记录;协议格式变更须处理版本。

## Goals / Non-Goals

**Goals:**
- **渐进回放**:客户端收到每页即应用,不必等全日志重组
- **确定性保序**:页面是确定性定稿日志的确定性子集,ReliableOrdered 保序,与直播客户端收敛一致
- **断点续传**:重连中途再掉线,`last_tick_consumed`(transport.rs:143 已在每次 flush 推进)保证自动从已应用点续传
- **页大小有界**:每页固定 tick 数(约 3KB,数分片),单页传输可控
- **协议扩展**:RelayClientMessage/RelayServerMessage 增变体,两端同仓升级

**Non-Goals:**
- 加密 / web-wasm / 权威服务器(后续 change)
- 改动可靠层分片(已工作,分页仍依赖它兜大页)
- 日志压缩(lockstep 低带宽下非瓶颈)
- **严格 relay 内存有界**:relay 必须持有未 ACK 数据以重传,内存 O(日志子集)不可避免(评审 W5);真实收益是客户端渐进交付

## Decisions

### D1: 推送式分页(偏离 D8 拉模型的记录)

- `ReconnectResponse` 改为**纯元数据**:`{ game_id, ruleset_version, seed, map_spec_hash, first_tick, total_ticks, page_count, players }`(移除 `ticks`)。
- 新增 `ReconnectPage { page_index, page_count, first_tick, ticks: Vec<TickCommands> }`,加入 `RelayServerMessage` 与 `NetworkEvent`。
- **推模型**:relay 回复元数据后,把 page_count 页按序 push 到 Control 通道。Control 是 ReliableOrdered → 页面按序到达,**无逐页请求 RTT**。
- **偏离理由(宪法 §16.3 要求记录)**:D8 原设计为「客户端逐页拉取」;推模型以可靠层保序取代拉取,省 N 次 RTT(重连高延迟链接收益大),客户端协议状态更少。拉模型的「客户端控制节奏」由可靠层 window+pacing 替代。

### D2: 按 tick 值分桶(CRITICAL #1 修复)

- 日志是 append 序,`log[i].tick` 不单调(try_finalize 可乱序定稿)。
- 页 i 覆盖 `[first + i*PAGE_TICKS, first + (i+1)*PAGE_TICKS)`,log 中 tick 落在该区间的**按值归入该页**,而非按 log 位置切片。
- `total_ticks` = log 中 `tick > last_tick_consumed` 的**计数**(非 tick 跨度)。
- 客户端 `apply_reconnect_page` 按 tick 键入 HashMap,页内/页间顺序由覆盖完整性保证。
- `PAGE_TICKS = 32`(约 3.2KB/页 → 约 4 个分片,可控)。

### D3: 限定范围 clear + system 顺序(CRITICAL #2 修复)

- 重连会话重入 `ctx.clients` 后,`broadcast()` 会把请求之后新定稿的 tick 发给它;该 broadcast 可与元数据同帧到达。
- 修复:`apply_reconnect` 的 `relay_buffer.clear()` 改为**仅移除 `tick < first_tick` 的陈旧项**;页面区间 `[first, first+total)` 由页面填充,`> first+total` 的实时 tick 保留。页面与 broadcast 覆盖分区不重叠(页面止于请求时最后定稿,broadcast 覆盖其后),无缺口无重复。
- 同时显式固定 `reconnect_recovery_system` 在 `network_poll_system` **之前**运行(两者均在 SimulationTickSet 前)。

### D4: 元数据校验 + 页游标(跨会话 stale 防御)

- 客户端以「页游标」跟踪已应用页:新元数据**重置游标**,且要求 `page.page_count == metadata.page_count` 才接受。
- 拒绝:乱序 `page_index`、重复页、`page_index >= page_count` 越界、页头 `page_count` 与元数据不符(旧会话残留页)。
- `apply_reconnect_page` 防御:拒绝 `tick <= last_tick_consumed`。

### D5: 边界处理

- `total_ticks = 0`(断线极短/无新 tick):`page_count = 0`,客户端收到元数据后**立即标记重放完成**,不得等待页到达。
- `page_count = 1`:退化单页,等价旧单消息路径。
- `frozen` 状态:`handle_reconnect` 返回 `Error`(冻结/超时是死路径——`check_freeze_timeout` 无调用方,防御性处理,不留新入口)。

### D6: 分批推页

- `handle_message` 不一次循环 push 全部页(会一次构建全部页,内存同现状;且受 poll 单次 10 数据报上限 + 10ms 循环限制)。
- relay 按 `ReliableSocket` 可观测 in-flight/已 ACK 进度**分批推页**(如每轮数页),维持管道流动且不冲刷内存。
- 客户端随收随应用,天然流水线。

### D7: 协议版本处理(宪法 §20.1)

- 消息枚举增 `ReconnectPage` 变体、`ReconnectResponse` 改元数据——两端同仓升级,无混合版本运行;无需改 reliable_udp 帧 MAGIC。
- 重放兼容仍由既有 `ruleset_version` 门控(apply_reconnect 校验),不新增字段。

### 宪法约束落地

- **分层**:全部改动在 bevy_adapter(network.rs / relay_core.rs / transport.rs),不动 simulation;RelayServer 仍是纯状态机。
- **确定性**:页面是确定性定稿日志的纯函数子集;ReliableOrdered 保序与 lockstep 不冲突(lockstep 约束仿真消费顺序,非传输通道);页面与 broadcast 同 tick 防御性去重(同一投影,覆盖安全)。
- **NoOp 兜底**:NoOp 注入在 try_finalize(纯函数),分页只读定稿后日志,不改变 NoOp 语义;重连再掉线 → 超时定稿 → NoOp → 日志增长 → 断点续传。
- **白名单**:仅 u32 页码 + TickCommands(bevy_adapter 协议类型,包裹 simulation::GameCommand),无 f32/渲染概念。

## Risks / Trade-offs

- [日志乱序 → 页内缺口 driver 死等] → D2 按 tick 值分桶 + 乱序定稿分页单测
- [元数据应用吞实时 broadcast → 确定性缺口] → D3 限定范围 clear + system 顺序固定
- [跨会话 stale 页被误收] → D4 页游标 + page_count 一致性校验
- [推模型与已批准 D8 拉模型分歧] → 本设计显式记录偏离理由(§16.3)
- [长断线 e2e 真实 UDP + sleep flaky] → 假 DatagramChannel + 注入时钟确定性构造;保留一个非 CI 门禁的真实 UDP 冒烟
- [relay 内存非严格有界] → 明确为设计取舍(可靠传输必须持有未 ACK 数据),目标改为渐进交付

## Migration Plan

1. 测试先行:分页边界/序单测(network.rs 状态机级,零 sleep)+ 客户端渐进应用单测
2. 协议变更:ReconnectResponse 元数据化 + ReconnectPage 变体(RelayServerMessage/NetworkEvent)
3. relay 侧:handle_reconnect 分桶分页 + relay_core 分批推页
4. 客户端:udp_session 逐页事件 + apply_reconnect/apply_reconnect_page 拆分 + reconnect_recovery_system 顺序固定
5. 测试扩展:reconnect_catchup 多页化 + 确定性长断线回放 hash 收敛 + 续传/竞态/退化单测
6. 回滚:分支级 revert

## Open Questions

- PAGE_TICKS 最优值(32 初值,页内分片数/进度粒度权衡)
- 分批推页的批大小与 in-flight 观测接口
- 页内防御性去重是否需要(可靠层已保证帧级去重,页面级是否仍需)

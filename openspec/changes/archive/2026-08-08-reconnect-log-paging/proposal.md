## Why

Change 2 已交付可靠 UDP 传输层,但重连恢复仍以**单条全量 ReconnectResponse** 返回断点后全部命令日志:客户端必须等完整消息重组后才交付任意 tick,长断线(数分钟 → 几百 KB)时无渐进回放、relay 一次性构建整个 blob。**功能正确但无渐进性**——这是 Change 2 verify 记录的已知限制(D8 分页,后续优化)。本 change 落地该设计:元数据 + 分页推送,客户端边收边放。

## What Changes

- **BREAKING(协议)**:`ReconnectResponse` 由「含 ticks」改为**纯元数据** `{ game_id, ruleset_version, seed, map_spec_hash, first_tick, total_ticks, page_count, players }`;新增 `ReconnectPage { page_index, page_count, first_tick, ticks: Vec<TickCommands> }`(RelayServerMessage + NetworkEvent)。两端同仓升级,无混合版本。
- **推模型分页**:relay 回复元数据后,把 `page_count` 页按序 push 到 Control 可靠通道(ReliableOrdered 保序,无逐页请求 RTT)。
- **按 tick 值分桶**:页面按 `[first + i*PAGE_TICKS, first + (i+1)*PAGE_TICKS)` 值区间归桶(日志 append 序非 tick 序),`total_ticks` = log 中 `tick > last_tick_consumed` 计数。
- **限定范围 clear**:`apply_reconnect` 只移除 `tick < first_tick` 陈旧项,不吞页面范围外的实时 broadcast。
- **渐进应用**:客户端每收一页即 `apply_reconnect_page` 灌入 relay_buffer,driver 按序消费;`last_tick_consumed`(每次 flush 已推进)保证断点续传。
- **防御校验**:页游标 + `page.page_count == metadata.page_count`;拒绝乱序/重复/越界页。
- **边界**:`total_ticks=0` 立即完成;`page_count=1` 退化单页;`frozen` 返回 Error。

无 simulation 改动;确定性回放语义不变。

## Capabilities

### New Capabilities

(无——分页是 network-reconnect 的协议演进,不新增 capability)

### Modified Capabilities
- `network-reconnect`: ReconnectResponse 元数据化 + ReconnectPage 分页推送,替代「全量日志 + MTU 分片」;客户端渐进应用 + 页游标校验

## Impact

- **network.rs**:`ReconnectResponse` 去 ticks 加元数据字段;新增 `ReconnectPage`;`handle_reconnect` 改分桶分页(`PAGE_TICKS=32`);`apply_reconnect` 拆为元数据校验 + `apply_reconnect_page`
- **relay_core.rs**:`handle_message` Reconnect 分支回复元数据后分批推页(D6 按 in-flight 分批)
- **transport.rs**:`udp_session` 处理 ReconnectPage 事件;`reconnect_recovery_system` 显式排在 `network_poll_system` 之前;`resp.ticks.len()` 引用同步改
- **测试**:network.rs 分页边界/序单测、客户端渐进应用单测、reconnect_catchup 多页化、确定性长断线回放(假 DatagramChannel + 注入时钟)
- **文档**:specs/network-reconnect delta 更新;落地后 sync 到 main specs

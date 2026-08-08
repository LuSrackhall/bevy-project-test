## Why

当前联机基于 TCP(relay 内嵌为主),存在队头阻塞(一个慢客户端拖住所有人)。目标 8 人以上 + 未来中央服务器 + IPv6,需要传输层消除队头阻塞、自主可控、IPv6 天然支持。经 4 子 agent 多维度审查(代码架构/宪法合规/网络协议/测试策略),确认自写可靠 UDP 是最优路径:完整可靠层(无 AIMD,低带宽固定 pacing)、relay 会话管理、MTU 分片、测试注入层。

## What Changes

- **新建 `reliable_udp` 模块**:自写可靠层(seq/cumulative ACK/滑动窗口/RTO 固定/去重/MTU 分片/固定 pacing),客户端 transport + relay 复用。抽象 `DatagramChannel` trait(真 UDP + 内存 netem 假通道),`ReliableSocket` 输出连接状态事件(Connected/Dead)
- **三通道**:ReliableOrdered(tick 命令)/ ReliableOrdered(控制:JoinGame/Reconnect/LobbyReady)/ Unreliable(心跳),防控制重传 HoL 阻塞 tick
- **无 AIMD**:lockstep 低带宽(几十 KB/s)下丢包≠拥塞,AIMD 减窗会拖停游戏;固定速率 pacing
- **重传降级追平**:重传耗尽 → 走 `apply_reconnect` 追平(宁追平不等包,不阻塞游戏)
- **relay 会话管理**:UDP 单 socket 共享 recv 按源地址去复用 → 会话表;心跳超时清扫触发 `on_disconnect`
- **重连日志传输**:`ReconnectResponse` 全量日志经 MTU 分片(IPv6 ≤1232)可靠传输(短断线正确);长断线按 tick 分页拉取为后续优化
- **IPv6 dual-stack**:bind `[::]`,IPv4-mapped 归一化;发现层 v4 广播保留独立 socket
- **全量迁移删 TCP**:transport.rs / relay_core / session_host / relay CLI + 测试迁移;`spawn_network_client` 改为"等到 GameJoined"

无 BREAKING(消息协议 `RelayClientMessage`/`RelayServerMessage` + bincode 不变,relay 状态机不变,仅换底层传输)。

## Capabilities

### New Capabilities
- `reliable-udp-transport`: 自写可靠 UDP 传输层 —— reliable_udp 模块(seq/ACK/窗口/重传/分片)+ 三通道 + 无拥塞控制 + DatagramChannel 测试注入层

### Modified Capabilities
- `network-reconnect`: 重连日志可靠传输(ReconnectResponse 全量 + MTU 分片;长断线分页为后续优化)
- `relay-server`: relay 端 UDP 会话管理(按源地址去复用会话表 + 心跳超时清扫触发 on_disconnect)

## Impact

- **bevy_adapter**: 新增 `reliable_udp` 模块(含 DatagramChannel trait + netem 假通道);`transport.rs`(客户端 TCP→UDP)、`relay_core.rs`(relay TCP→UDP + 会话表)、`session_host/thread.rs`、`lib.rs`(系统注册)
- **relay**: `src/lib.rs`(CLI 适配 UDP)、`tests/*`(5 个测试迁移)
- **测试**: netem 单测(丢包/乱序/重传/重复/分片/seq 回绕)+ 丢包集成测试(5-20% 丢包双端 hash 收敛)+ 乱序定稿回归 + 心跳掉线测试
- **文档**: ADR(传输层选型:自写可靠 UDP vs QUIC/renet2)
- **移除**: TCP 相关代码(`TcpListener`/`TcpStream`/长度前缀帧)

## Context

当前联机为 TCP + 长度前缀 bincode,relay 内嵌为主(`session_host`)。Change 1 已解锁 8 人以上正确性 + 掉线重连(基于 TCP)。目标:传输层改造为**自写可靠 UDP**,消除 TCP 队头阻塞,为中央服务器 + IPv6 铺路。经 4 个子 agent 多维度审查(代码架构/宪法合规/网络协议/测试策略),设计已整合全部关键发现。

## Goals

- 新建 `reliable_udp` 模块:自写可靠层(seq/ACK/滑动窗口/超时重传/去重/MTU 感知分片),客户端 transport + relay 服务器复用
- transport.rs / relay_core / session_host / relay CLI 全量迁移到 UDP,**删除 TCP**
- UDP socket **dual-stack**(bind `::`),同时服务内嵌回环 + 远程
- 可靠层可独立单测:通过 `DatagramChannel` trait + 内存 netem 假通道 + 虚拟时钟,确定性模拟丢包/乱序/重传/重复/分片
- 协议(`RelayClientMessage`/`RelayServerMessage` + bincode)、relay 状态机、确定性保持不变
- 宪法合规:双向 ReliableOrdered、NoOp 降级兜底、乱序定稿回归测试

## Non-Goals

- 内存通道(内嵌本地走 UDP 回环,延迟 <1ms 无感)
- 加密(自写可靠层无 TLS,后续 Change 可加)
- 拥塞控制(AIMD 在 lockstep 低带宽下不适用,见 D3)
- web/wasm(桌面优先)

## Decisions

### D1 — 独立 `reliable_udp` 模块 + `DatagramChannel` trait

封装可靠连接,客户端 transport + relay 服务器复用。**抽象 `DatagramChannel` trait**:真实 `tokio::UdpSocket` 实现 + 测试用内存 netem 假通道(脚本化故障注入:丢包/乱序/重复/分片/丢 ACK)。`ReliableSocket` 输出**连接状态事件**(`Connected`/`Dead`),供 transport 触发重连、relay_core 调 `on_disconnect`(recv 返回 Option 无法区分"无数据"vs"连接死亡")。拆 sender/receiver 子模块(seq/ACK/窗口/重传/分片)。

### D2 — 三通道

- `ReliableOrdered`(tick 命令帧,必达有序,主通道)
- `ReliableOrdered`(控制:JoinGame/Reconnect/LobbyReady)——与 tick 分通道,防控制消息重传 HoL 阻塞 tick
- `Unreliable`(心跳)

### D3 — 可靠机制(**无 AIMD**)

- 单调 seq + **cumulative ACK** + 滑动窗口(发送未 ACK 上限)+ 去重(接收方按 seq,防重复)
- RTO **自适应**(SRTT/Karn,下限保护)+ 重传上限
- **重传耗尽 → 降级走 `apply_reconnect` 追平**(宁追平不等包,不阻塞游戏)——防止命令延迟破坏 lockstep
- **MTU 感知分片**(IPv6 最小 MTU 1280 → 有效载荷 ≤1232),分片载体主要是重连大日志
- **固定速率 pacing**(无拥塞控制:AIMD 在 8 人×20Hz≈几十 KB/s 带宽下减窗会拖停 lockstep,丢包≠拥塞)
- seq 回绕防护(窗口 ≤ 半 seq 空间)

### D4 — IPv6 dual-stack

- relay bind `[::]:port`(dual-stack 收 IPv4/IPv6),处理 **IPv4-mapped 地址归一化**(`::ffff:a.b.c.d`)
- 内嵌本地连回环 `::1`,远程连 relay 广播的地址
- **发现层 v4 广播保留独立 socket**(不能走 dual-stack);beacon UDP 与游戏 UDP 端口分离避免冲突

### D5 — 全量迁移(删 TCP)

- transport.rs 的 TCP 读写循环 → UDP + 可靠层;`spawn_network_client` 的"阻塞至 TCP 建立"语义改为"**等到 GameJoined**"
- relay_core 的 `TcpListener`/`handle_client` → UDP socket + 每连接可靠状态(见 D7)
- `session_host`/`relay` CLI 适配
- 测试迁移 5 个(`network_e2e`/`network_move_e2e`/`relay integration`/`two_client_sync`;`reconnect_catchup` 无 socket 无需迁)+ 新增**丢包集成测试**(5-20% 丢包双端最终同 tick 同 hash,证明乱序重传后命令有序归位)

### D6 — 心跳/超时检测(替代 TCP 断开检测)

- 心跳走 Unreliable 通道,客户端周期发(如 500ms)
- 超时阶梯:心跳缺失超阈值 → 客户端触发重连;relay 判定掉线 → 标 `Disconnected`(席位保留)靠 NoOp 推进(衔接 Change 1 掉线重连 ADR 0009)
- **防误判**:超时阈值需大于正常抖动(心跳间隔 × N),不误杀活客户端(测试覆盖)

### D7 — relay 端会话管理(新增,架构 P1)

- UDP 单 socket 收所有客户端:**共享 recv 循环按源地址去复用** → 会话表(每客户端可靠连接状态)
- JoinGame 前未认证报文的处理路径
- **心跳超时清扫**:空闲会话超时触发 `on_disconnect`(无 read error 时 relay 端靠此驱动掉线)
- 会话表与 Change 1 席位复用/重连竞态对齐(ADR 0009)

### D8 — 重连大日志分页传输(新增)

- `ReconnectResponse` 含断点后全部命令日志,断线数分钟 = 几千 tick × ~100B = **几百 KB**,超 64KB datagram 与 MTU
- 日志**按 tick 分页可靠传输**(客户端逐页拉取),配合 D3 分片

### 宪法约束(双向可靠 + 降级)

- **双向 ReliableOrdered**:client→relay(tick 命令)+ relay→client(broadcast 广播)都必须走可靠通道——否则各客户端缺不同 tick → desync
- **NoOp 降级兜底**:可靠层失效 → 重传耗尽 → 丢弃 → relay 超时定稿 → NoOp 注入(§3.2)
- **乱序定稿回归测试**:UDP 乱序使高 tick 先定稿成常态,衔接 Change 1 修的 try_finalize(扫描 log),需回归验证 + 客户端 relay_buffer 按序消费 + 乱序定稿追平测试

## Risks / Trade-offs

- [可靠层实现 bug(丢包/重传/窗口/分片)] → `DatagramChannel` netem 单测(脚本化故障注入 + 虚拟时钟,零 sleep,确定性复现)
- [重连日志超 UDP 上限] → D8 分页 + D3 MTU 分片
- [重传延迟吞命令] → 重传上限 + 降级追平 + jitter 调优
- [掉线误判] → D6 心跳超时阶梯 + 防假阳性测试
- [relay 会话管理复杂度] → D7 会话表 + 空闲清扫,单测覆盖
- [测试 flakiness(真实 UDP)] → DatagramChannel 假通道主导单测;真实 socket 集成测试用"重发 JoinGame 至 ACK"作就绪信号,替代固定 sleep

## 后续(独立 change,不在本次)

- 内存通道(内嵌本地零序列化,若内嵌长期为主)
- 加密(可靠层加 TLS 或转 QUIC)
- web/wasm 传输(WebTransport)
- 权威服务器(宪法 §9.3,防作弊)

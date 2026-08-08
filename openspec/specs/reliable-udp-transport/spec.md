# reliable-udp-transport Specification

## Purpose

自写可靠 UDP 传输层,替代 TCP 承载联机 RTS 消息。提供 seq/cumulative ACK/滑动窗口/超时重传/去重/MTU 分片的可靠有序交付,消除 TCP 队头阻塞,为中央服务器 + IPv6 铺路。

## Requirements

### Requirement: Reliable ordered message delivery

可靠 UDP 层 SHALL 保证可靠有序消息交付:发送方单调 seq,接收方按序缓存、去重、回复 cumulative ACK,超时重传。消息必达且有序(ReliableOrdered 语义)。

#### Scenario: dropped datagram is retransmitted

- **WHEN** 一个可靠数据报在传输中被丢弃(模拟丢包)
- **THEN** 发送方超时重传,接收方最终收到该消息,且无重复交付

#### Scenario: out-of-order datagrams are reordered

- **WHEN** 两个可靠数据报乱序到达
- **THEN** 接收方按 seq 缓存并按序交付,不产生乱序消息

#### Scenario: duplicate datagram is dropped

- **WHEN** 同一 seq 的数据报重复到达
- **THEN** 接收方丢弃重复,不重复交付

### Requirement: DatagramChannel abstraction for fault injection

可靠层 SHALL 抽象 `DatagramChannel` trait,支持真实 UDP 与测试用内存通道(可脚本化注入丢包/乱序/重复/分片,并支持虚拟时钟)。可靠层的丢包/重传/窗口逻辑 SHALL 可确定性单测(零 sleep)。

#### Scenario: fault injection deterministically reproduces

- **WHEN** 测试通过内存通道注入"丢弃第 3 包、重排第 4/5 包、重复第 7 包"
- **THEN** 可靠层在虚拟时钟驱动下确定性处理,无需真实网络或 sleep

### Requirement: Connection state events

`ReliableSocket` SHALL 输出连接状态事件(`Connected`/`Dead`),而非仅返回 Option——调用方需区分"无新消息"与"连接死亡",以触发重连(客户端)或掉线清扫(relay)。

#### Scenario: dead connection triggers disconnect

- **WHEN** 心跳超时判定连接死亡
- **THEN** relay 输出 Dead 事件并触发 `on_disconnect`;客户端输出 Dead 事件并触发重连

### Requirement: Three channels isolation

可靠层 SHALL 提供三通道:ReliableOrdered(tick 命令)/ ReliableOrdered(控制:JoinGame/Reconnect/LobbyReady)/ Unreliable(心跳)。控制消息重传 SHALL NOT 阻塞 tick 通道。

#### Scenario: control retransmission does not block ticks

- **WHEN** 一条控制消息(如 JoinGame)需要重传
- **THEN** 同期 tick 通道的命令仍正常交付,不受控制通道重传阻塞

### Requirement: Fragmentation and reassembly

帧超 MTU 有效载荷(IPv6 最小 MTU 1280 → ≤1232 字节)时 SHALL 分片传输并按 `(msg_id, frag_idx, frag_total)` 重组。分片主载体为重连大日志。

#### Scenario: oversized frame is fragmented and reassembled

- **WHEN** 一条消息超 MTU(如大 ReconnectResponse 日志)
- **THEN** 发送方分片,接收方收齐后重组为完整消息交付

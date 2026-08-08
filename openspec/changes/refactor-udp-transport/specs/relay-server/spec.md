## ADDED Requirements

### Requirement: Relay manages UDP sessions by source address

relay SHALL 维护每客户端 UDP 会话表:单 socket 共享 recv 后按源地址去复用报文,路由到对应会话的可靠状态。JoinGame 建立完整会话,端口变化时重发 JoinGame 更新源地址。

#### Scenario: session tracked by source address

- **WHEN** 客户端从某源地址发 JoinGame 建立会话
- **THEN** relay SHALL 将会话记录在会话表(源地址 → 会话),后续该地址报文路由到对应会话

#### Scenario: port change updates session address

- **WHEN** 客户端因 NAT rebinding 从新端口重发 JoinGame
- **THEN** relay SHALL 更新会话的源地址,继续识别同一玩家

### Requirement: Heartbeat timeout triggers disconnect

relay SHALL 通过心跳超时判定客户端掉线(替代 TCP 连接断开检测):会话的 `last_seen` 超阈值(如心跳间隔 × N)后,标记 `Disconnected`(席位保留,靠 NoOp 推进)并触发 `on_disconnect`。超时阈值 SHALL 大于正常抖动,避免误判活客户端。

#### Scenario: heartbeat timeout marks disconnected

- **WHEN** 客户端心跳超时(超过阈值未收到)
- **THEN** relay SHALL 判定该会话掉线,标记 `Disconnected`(席位保留)并触发 `on_disconnect`

#### Scenario: jitter does not falsely kill a live client

- **WHEN** 客户端因网络抖动心跳延迟(但在阈值内)
- **THEN** relay SHALL NOT 误判掉线,会话保持活跃

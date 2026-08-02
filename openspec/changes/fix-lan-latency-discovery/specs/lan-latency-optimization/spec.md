## ADDED Requirements

### Requirement: TCP_NODELAY

所有客户端↔relay TCP 连接必须禁用 Nagle 算法。

#### Scenario: 客户端连接
- **WHEN** 客户端 TCP 连接到 relay 成功
- **THEN** 必须调用 `set_nodelay(true)`

#### Scenario: relay 接受连接
- **WHEN** relay 接受客户端连接
- **THEN** 必须调用 `set_nodelay(true)`

### Requirement: 子网广播

beacon 必须同时发送到 255.255.255.255 和从 LAN IP 推导的子网广播地址。

#### Scenario: 多网卡主机
- **WHEN** 主机有多个网卡，LAN IP 为 192.168.1.157
- **THEN** beacon 必须发送到 192.168.1.255 和 255.255.255.255

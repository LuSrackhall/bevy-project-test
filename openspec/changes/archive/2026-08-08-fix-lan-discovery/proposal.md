## Why

用户实测（rc17，Mac + Win 同局域网）：互不可见对方创建的房间，无法联机。根因：主机（创建房间方）的发现 beacon 绑定 `0.0.0.0:9876` 失败——`LanDiscoveryListener`（浏览房间列表页时）已持有同一端口，beacon 绑定 `EADDRINUSE` 后静默禁用，主机从不广播房间。Mac/Win 对称受影响。

## What Changes

- **beacon socket 改绑 `0.0.0.0:0`（临时端口）+ `set_broadcast(true)`**：beacon 只需向目标端口 9876 发送，源端口无关紧要（监听器按包内 `relay_id` 匹配）；`0.0.0.0:0` 彻底避免与 `LanDiscoveryListener`（0.0.0.0:9876）争端口。
- 发送目标不变（子网广播 + `255.255.255.255` + 回环，均到 `:9876`）；监听器（lan.rs）不变。
- **附带消除**：Host 转 Lobby 时 OnExit 移除 listener 的时序竞争。

无协议格式变更（bincode / relay_id 匹配不变）；无 simulation 改动。

## Capabilities

### New Capabilities

(无)

### Modified Capabilities
- `lan-discovery`: UDP beacon 广播从绑定 9876 改为临时端口，避免与浏览监听器争端口（修复跨机互不可见）

## Impact

- **session_host/thread.rs**: beacon socket bind `0.0.0.0:0`（原 0.0.0.0:9876）
- **lan.rs**: 不变（可选：加 `start_on(port)` 便于测试隔离）
- **测试**: 生产路径 e2e（LanDiscoveryListener + ThreadRelayRuntime::start → drain 断言收到包）+ 源端口探针 + packet 单测 + 回归
- **文档**: 防火墙提示（入站 UDP 9876 + relay 临时端口需放行）

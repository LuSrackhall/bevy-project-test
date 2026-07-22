## Why

创建 LAN 房间后，`ThreadRelayRuntime` 不广播 UDP 发现信标，导致房间从不出现在任何客户端的房间列表中。当前 `lan-discovery` 规范已要求 relay 广播 UDP 信标，但 `ThreadRelayRuntime` 占位实现未满足此要求。

## What Changes

- `run_local_relay` 增加 UDP beacon 广播逻辑：每 3 秒广播 `LanDiscoveryPacket` 到 LAN
- 签名改为显式传递 `RelayId` + `RoomMetadata`，保证信标内容与房间元数据一致
- 所有 UDP 错误仅 log，不阻塞房间创建
- TCP 绑定后立即通知主线程，保持当前不阻塞语义

## Capabilities

### New Capabilities
（无 — 此变更实现 `lan-discovery` 规范中已有的信标广播要求）

### Modified Capabilities
- `lan-discovery`: 增加 `ThreadRelayRuntime` 的 UDP 信标广播实现

## Impact

| 范围 | 文件 | 说明 |
|---|---|---|
| 仅内部实现 | `crates/bevy_adapter/src/session_host/thread.rs` | `run_local_relay` 函数体重写 |
| 参数适配 | `crates/bevy_adapter/src/session_host/thread.rs` | `run_local_relay` 签名变更 |
| 无 | `crates/bevy_adapter/src/session_host/controller.rs` | 调用侧适配新签名（传递 relay_id） |
| 无 | trait 接口 | 不变 |

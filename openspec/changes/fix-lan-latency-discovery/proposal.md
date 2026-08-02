## Why

联机存在两个问题：Nagle 算法导致命令"有时卡"，beacon 单一全局广播导致 Windows 主机对 Mac 不可见。

## What Changes

- 所有 TCP socket 启用 `set_nodelay(true)`（禁用 Nagle）
- beacon 额外发送到从 `detect_lan_ip()` 推导的 /24 子网广播地址

## Capabilities

### New Capabilities
- `lan-latency-optimization`: 降低 LAN 联机延迟 + 提高房间发现可达性

### Modified Capabilities
无。

## Impact

- `crates/bevy_adapter/src/transport.rs`（两处 connect）
- `crates/bevy_adapter/src/relay_core.rs`（accept）
- `crates/bevy_adapter/src/session_host/thread.rs`（beacon）

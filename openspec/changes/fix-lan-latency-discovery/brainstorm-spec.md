## Context

用户报告两个联机问题：
1. **响应性不足**：移动指令虽已改善，但主机"有时会卡"，加入方更明显。根因：所有 TCP socket 未设置 `set_nodelay(true)`，Nagle 算法 + Windows 延迟 ACK 导致小包（PlayerTickFrame 仅约 30 字节）被缓冲 40-200ms。
2. **Windows 主机不可见**：Windows 当主机时 Mac 看不到房间。根因：beacon 只发 `255.255.255.255:9876`，Windows 多网卡时全局广播可能只走一个接口（非 Mac 所在子网）。

## Goals / Non-Goals

**Goals:**
- 禁用 Nagle 算法，消除 40-200ms 小包延迟
- beacon 发送到子网广播地址（从 detect_lan_ip 推导），提高跨网卡可达性
- 保持 255.255.255.255 全局广播作为回退

**Non-Goals:**
- 不降低 input_delay（降低会增加丢命令风险，实测后再定）
- 不新增外部依赖
- 不改变协议

## Decisions

### 1. TCP_NODELAY

三个 socket 位置调用 `set_nodelay(true)`：
- `transport.rs` 两处 `TcpStream::connect` 成功后
- `relay_core.rs` `handle_client` accept 后

### 2. 子网广播

从 `detect_lan_ip()` 推导 /24 子网广播地址（如 192.168.1.157 → 192.168.1.255），beacon 额外发送到该地址，保留 255.255.255.255。

### 不选择 if-addrs 依赖

if-addrs 需新增 ADR 和依赖。/24 假设适用于常见家庭/办公 LAN，本场景足够。若后续发现多子网/VLAN 场景，再升级。

### 不降低 input_delay

input_delay=3 是丢命令的容错缓冲。降低会增加极端延迟下丢命令风险，与"不接受卡顿"目标冲突。NODELAY 落地实测后再定。

## Risks / Trade-offs

- [Risk] /24 子网假设不适用非 /24 网络 → 保留 255.255.255.255 回退
- [Risk] NODELAY 后 relay_write 分两次写（len+body）变成两包 → 可接受（~100 包/秒，可忽略），后续可合并 buffer

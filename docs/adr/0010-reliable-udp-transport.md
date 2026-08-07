# ADR 0010: 传输层改为自写可靠 UDP

## 状态

**Date**: 2026-08-08
**Status**: Accepted（实现于 refactor-udp-transport）

## 背景

原传输层为 TCP + 长度前缀 bincode。存在队头阻塞（一个慢客户端拖住所有人）。目标 8 人以上对局 + 未来中央服务器 + IPv6，需要消除队头阻塞、自主可控、IPv6 天然支持。

## 决策

传输层改为**自写可靠 UDP**，新建独立 `reliable_udp` 模块：

- **可靠机制**：单调 seq + cumulative ACK + 滑动窗口 + RTO 自适应（SRTT/Karn）+ 重传上限 + 去重（回绕安全判断）+ MTU 感知分片（IPv6 ≤1232）
- **三通道**：ReliableOrdered（tick 命令）/ ReliableOrdered（控制：JoinGame/Reconnect/LobbyReady）/ Unreliable（心跳）
- **无拥塞控制**：lockstep 带宽极低（8 人×20Hz≈几十 KB/s），丢包≠拥塞，AIMD 减窗会拖停游戏；固定 pacing
- **IPv6 dual-stack**：relay bind `[::]`，v4-mapped 地址处理；发现层 v4 广播独立 socket
- **会话管理**：relay 单 socket 共享 recv 按源地址去复用 → 每客户端 ReliableSocket 会话；心跳超时清扫触发 on_disconnect
- **可测性**：`DatagramChannel` trait 抽象真实 UDP 与内存 netem 假通道（脚本化故障注入 + 虚拟时钟），可靠层可确定性单测

## 候选对比

- **renet2**（github.com/UkoeHB/renet2）：个人 fork 项目（59★）、0.x、原版 renet 停更——作者可信度与维护风险（与用户对 renet2 的质疑一致）
- **QUIC 库**（quinn/quiche/s2n-quic）：quinn 纯 Rust 且 bevy_quinnet 有游戏先例，但引入第三方依赖、API 面向通用流、偏重；quiche/s2n-quic 强制 C 工具链、无游戏用例
- **自写可靠 UDP（选定）**：完全自主、零依赖、延迟精细可控、API 贴合游戏消息

## 影响

- transport.rs / relay_core / session_host / relay CLI 全量迁移，删除 TCP（TcpStream/TcpListener/长度前缀帧）
- 测试迁移到 UDP；可靠层单测（netem 丢包/乱序/重传/去重/分片）+ 集成测试
- 掉线检测改为心跳（1.5s 超时），替代 TCP 连接断开检测

## 关联

- specs/reliable-udp-transport（new）、specs/network-reconnect、specs/relay-server（delta specs）
- 宪法 §3.1/§3.2（lockstep 六步、NoOp 补齐）、§9.1（同套仿真代码）、§5.5（分层）

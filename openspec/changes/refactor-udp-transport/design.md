## Context

Change 2 目标:传输层从 TCP 改造为自写可靠 UDP。高层设计已由 brainstorm-spec.md 批准(D1-D8 + 宪法约束),经 4 子 agent 多维度审查整合。本文深入实现架构、数据流、模块边界、错误处理与测试策略。宪法约束:分层(§1)、simulation 白名单(§1.4)、确定性(§0.1)、lockstep 六步(§3)、同套仿真代码(§9)、NoOp 兜底(§3.2)。

## Goals / Non-Goals

**Goals:**
- `reliable_udp` 模块可独立单测(丢包/乱序/重传/重复/分片,零 sleep 确定性)
- 全量迁移删 TCP,消息协议/relay 状态机/确定性不变
- 双向 ReliableOrdered + NoOp 降级 + 乱序定稿回归

**Non-Goals:**
- 内存通道、加密、AIMD 拥塞控制、web/wasm

## Decisions

### D1: `reliable_udp` 模块结构

```
reliable_udp/
  mod.rs          — ReliableSocket(核心:seq/窗口/RTO/去重/分片,sender/receiver 内联)
  protocol.rs     — 帧格式(seq/kind/frag 编码解码)
  channel.rs      — DatagramChannel trait
  channel_udp.rs  — tokio UdpSocket 实现
  channel_netem.rs— 测试假通道(脚本化故障注入 + 虚拟时钟)
```

**`DatagramChannel` trait**(测试基石):
```rust
#[async_trait]
trait DatagramChannel: Send {
    async fn send_to(&mut self, buf: &[u8], to: SocketAddr) -> io::Result<()>;
    async fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
    // 测试用虚拟时钟钩子
    fn now(&self) -> Instant;  // netem 通道用注入时钟
}
```

**`ReliableSocket` API**:`send_reliable(msg)` / `send_control(msg)` / `send_unreliable(msg)` / `recv() -> Option<Incoming>`。输出**连接状态事件**:`Incoming::Message(msg)` / `Incoming::Connected` / `Incoming::Dead`(替代 recv 返回 Option 无法区分"无数据"vs"连接死亡"的缺口)。

### D2: 三通道

```rust
enum Channel { Tick, Control, Heartbeat }  // Tick/Control 可靠,Heartbeat 不可靠
```
- 每通道独立 seq 空间(防跨通道 seq 干扰)
- tick 命令走 Tick 通道(ReliableOrdered);JoinGame/Reconnect/LobbyReady 走 Control 通道;心跳走 Heartbeat(Unreliable,不重传)

### D3: 可靠机制

- **seq + cumulative ACK**:发送方每帧带单调 seq;接收方回 cumulative ACK(确认至某 seq);发送方按 ACK 推进窗口
- **滑动窗口**:发送未 ACK 上限(如 32 帧),满则 pacing 等待
- **RTO 固定**:200ms 初始 + 重传上限(5 次)。SRTT/Karn 自适应与指数退避留后续(当前 LAN 低延迟可用)
- **重传耗尽降级**:超上限后**不阻塞游戏**——置 dead 标志并丢弃该帧,transport 检测 is_dead 触发 `apply_reconnect` 追平(宁追平不等包)
- **去重**:接收方按 seq 缓存,重复帧丢弃;seq 回绕防护(窗口 ≤ 半 seq 空间,seq 用 u32)
- **分片**:帧 > MTU 有效载荷(IPv6 ≤1232)时分片;每片带 (msg_id, frag_idx, frag_total);重组后交付。分片主载体是重连大日志
- **固定 pacing**:发送间隔 ≥ 某阈值(如 1ms),无拥塞控制(AIMD 在低带宽下不适用)

### D4: IPv6 dual-stack

- relay: `UdpSocket::bind("[::]:port")`(dual-stack 收 IPv4/IPv6),`IPV6_V6ONLY=false`
- IPv4-mapped 地址(`::ffff:a.b.c.d`)归一化回 v4 比较(会话表 key)
- 内嵌本地连 `::1`,远程连 relay 广播地址
- 发现层 v4 广播**保留独立 UdpSocket**(v4 广播不能走 dual-stack v6 socket)
- beacon 与游戏 UDP 端口分离(beacon 用 9876,游戏用 relay 绑定端口)

### D5: 全量迁移

1. **relay_core**:`TcpListener` + `handle_client` → `UdpSocket` + D7 会话表。`run_relay` 签名从 `listener: TcpListener` 改为 `socket: UdpSocket`。
2. **transport.rs**:`spawn_network_client` 的 TCP connect + run_session(read/write loop)→ UDP + `ReliableSocket`。**"阻塞至 TCP 建立"语义改为"等到 GameJoined"**(UDP 无握手)。
3. **session_host/thread.rs**:`TcpListener::bind` → `UdpSocket::bind("[::]:0")`;beacon 独立。
4. **relay CLI**:`TcpListener` → `UdpSocket`。
5. **测试**:5 个 TCP 测试迁移(`network_e2e`/`network_move_e2e`/`relay integration`/`two_client_sync`;`reconnect_catchup` 无 socket 不迁)+ 新增丢包集成测试。

### D6: 心跳/超时

- 客户端周期(500ms)发 `Heartbeat`(Unreliable 通道)
- relay 会话表记录 `last_seen`;超时(如 3×500ms=1.5s)判定掉线 → `on_disconnect`(席位保留,衔接 ADR 0009)
- 客户端侧:超时未收到 relay 广播 → 触发重连(发 JoinGame + ReconnectRequest)
- 防误判:超时阈值 > 正常抖动;测试覆盖"延迟抖动不误杀活客户端"

### D7: relay 会话表

```rust
struct RelaySession {   // 每客户端一个
    addr: SocketAddr,   // 源地址(join 时注册,处理端口变化)
    socket: ReliableSocket,  // 每会话可靠状态
    player_id: u8,
    last_seen: Instant,
}
// relay 主循环:共享 recv → 按源地址查/建会话 → 分发给对应 ReliableSocket
// 心跳超时清扫:周期扫描 last_seen,超时 → on_disconnect + 移除
```
- JoinGame 前未认证报文:缓冲到最小会话(仅缓存源地址),等 JoinGame 建立完整会话
- 端口变化/NAT rebinding:JoinGame 重发时更新 addr

### D8: 重连日志分页(后续优化)

- 设计:`ReconnectResponse` 拆分首帧元信息 + 客户端逐页拉取 `ReconnectPage`。
- **实现状态**:当前 `handle_reconnect` 返回全量日志(短断线 < MTU 时正确,分片兜底);长断线(数分钟)大日志的分页拉取为后续优化,记录于已知限制。

### 宪法约束落地

- **双向 ReliableOrdered**:client→relay(tick)+ relay→client(broadcast)都走可靠通道;relay 广播失败(某客户端重传耗尽)→ 该客户端后续走重连追平
- **NoOp 降级**:可靠层失效帧丢弃 → relay 超时定稿 → NoOp 注入(§3.2)
- **乱序定稿回归测试**:UDP 乱序使高 tick 先定稿;验证 try_finalize(扫描 log,Change 1 修复)+ 客户端 relay_buffer 按序消费

## Risks / Trade-offs

- [可靠层 bug] → DatagramChannel netem 单测(脚本化故障注入 + 虚拟时钟,零 sleep 确定性复现)
- [重连日志超限] → D8 分页 + D3 MTU 分片
- [重传延迟吞命令] → 重传上限 + 降级追平 + jitter 调优
- [掉线误判] → D6 心跳超时阶梯 + 防假阳性测试
- [relay 会话复杂度] → D7 会话表 + 空闲清扫,单测覆盖
- [测试 flakiness] → netem 假通道主导单测;真实 socket 集成测试用"重发 JoinGame 至 ACK"就绪信号,替代固定 sleep

## Migration Plan

1. 测试先行:netem 假通道 + 可靠层单测(丢包/乱序/重传/重复/分片/seq 回绕/窗口停滞)
2. `reliable_udp` 模块(真 UDP 实现)
3. relay 端迁 UDP(会话表 + 心跳清扫)
4. 客户端 transport 迁 UDP(ReliableSocket + 等到 GameJoined)
5. 测试全迁 + 丢包集成测试 + 乱序定稿回归 + 心跳掉线测试
6. 删 TCP 代码
7. 回滚:分支级 revert

## Open Questions

- 心跳间隔/超时阈值具体值(500ms/1.5s 初值,调优)
- RTO 初始值/退避系数、重传上限(5 次初值)
- 重连分页大小(每页字节上限)
- pacing 速率下限
- 乱序定稿追平的 jitter 调优

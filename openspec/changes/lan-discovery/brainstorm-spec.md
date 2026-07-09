## Context

当前用户需手动输入 relay 地址和端口才能联机。LAN 发现允许同局域网用户自动发现彼此。三个子 Agent 审计确认了设计方向和安全约束。

## Goals / Non-Goals

**Goals:**
- `LanDiscoveryPacket` 自定义固定长度协议（非 bincode，避免反序列化风险）
- Relay 启动时广播 UDP 信标（空闲 3s，活跃 10s 退避）
- `LanDiscoveryListener` Bevy Resource + tokio 后台线程
- 主菜单服务器列表 UI + 直接连接

## Decisions

### D1：LanDiscoveryPacket（自定义序列化）

```rust
pub struct LanDiscoveryPacket {
    magic: [u8; 2],      // b"RT"
    version: u8,          // 1
    relay_port: u16,      // 网络字节序
    player_count: u8,
    current_players: u8,
    game_state: u8,       // 0=lobby, 1=playing
}
```

总长度固定 9 字节，无字符串、无变长字段、无 bincode 调用。

### D2：中继侧广播

UDP socket 绑定同一端口，`set_broadcast(true)`，每 3s 广播到 `255.255.255.255`。

### D3：客户端监听

`LanDiscoveryListener`（`Arc<Mutex<Vec<LanDiscoveryPacket>>>`），OnEnter(MainMenu) 启动 tokio 线程，OnExit 停止。

### D4：服务器列表

5s 过期，按 addr 去重，列表项点击直接触发 `NeedsGameReset::Network`。

## Risks

| Risk | Mitigation |
|------|-----------|
| 反序列化攻击 | 固定长度协议，无 bincode |
| 恶意 UDP 广播 | 验证 magic + 限制包大小 |
| tokio 线程泄露 | LanDiscoveryListener Drop + OnExit 清理 |

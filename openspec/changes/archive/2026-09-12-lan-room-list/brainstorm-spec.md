## Context

当前联机需要用户在终端手工指定 `--relay <ip>:<port> --player-id <id>`。已有 LAN UDP 发现框架（`LanDiscoveryListener`、`LanServers`），但只广播 relay 地址、未集成到 UI。

现有 GameState：`MainMenu → Lobby → Playing`，联机和单人逻辑混合在同一个菜单。

## Goals / Non-Goals

**Goals：**
- 主菜单分"单人模式"和"局域网模式"两个独立入口
- 局域网模式：房间列表（动态发现局域网所有房间）+ 创建房间 + 加入房间
- 用户全程不接触 IP/端口
- 支持多房间共存，同一局域网可见全部

**Non-Goals：**
- 断线重连 / Host Migration
- 单人模式房间配置（AI、阵营等）
- 公网大厅 / 匹配

## Invariants（不变式）

这些设计约束必须保持，违反将导致后续接口返工。

### I1: Room Owner ≠ Relay ≠ Player

三者是独立的概念。LAN MVP 中部署关系恰好是：

```
LocalSessionHost
└── owns RelayRuntime

RoomOwner
└── controls Room
```

UI 层只关心"房间"，不直接操作 relay。

### I2: 创建房间不是启动外部 relay 进程

抽象 `RelayRuntime` / `SessionHost` 生命周期接口。LAN 实现可以是嵌入 `start_relay()` 函数调用或子进程，但 UI 层依赖接口，不依赖 `std::process::Command`。未来 Dedicated Relay 不需要改 UI 和房间模型。

### I3: UDP Beacon 是快照，不是权威状态

发现包只做"广告"用途，包含：`room_id`、`protocol_version`、`relay_endpoint`、`room_name`、人数摘要、状态摘要。加入后的房间完整状态以 Relay 的权威响应为准。

### I4: player_id 由 Relay 分配

用户加入房间后，Relay 分配 `player_id` / slot。客户端不再需要 `--player-id` 参数。流程：

```
发现房间 → Connect(endpoint) → JoinRoom
 → Relay 分配 player_id / slot
 → 返回 RoomSnapshot（包含其他玩家信息）
 → Lobby
 → GameStarted
```

## Architecture Decisions

### AD1: 发现协议扩展

现有 `LanDiscoveryPacket` 扩展字段：

```rust
pub struct LanDiscoveryPacket {
    pub magic: u16,
    pub version: u16,
    pub room_id: u64,           // 新增：唯一房间标识（房主生成）
    pub protocol_version: u16,  // 新增：协议版本兼容检查
    pub room_name: String,      // 新增
    pub map_size: u8,           // 新增
    pub relay_port: u16,
    pub current_players: u8,
    pub max_players: u8,
    pub game_state: u8,         // 0=等待中 1=游戏中
}
```

`room_id` 由房主在创建房间时生成（随机 u64），用于列表去重和加入定位。

### AD2: Relay 端口由 OS 分配

绑定 `127.0.0.1:0`，通过 `local_addr()` 获取实际端口，写入发现包广播。避免端口冲突。

### AD3: 加入协议

客户端连接 relay 后，发送新的 `RelayClientMessage::JoinRoom`。Relay 返回 `RelayServerMessage::JoinedRoom { player_id, slots }`。之后等待房主触发 `GameStarted`（复用现有 `LobbyReady → GameStarted` 流程）。

### AD4: RelayRuntime 生命周期接口

```rust
trait RelayRuntime {
    fn start(&mut self) -> Result<u16, String>;  // 返回实际端口
    fn stop(&mut self);
}
```

LAN 实现：内部调用 `start_relay(port: 0, ...)`。默认可作为当前 relay crate 的封装。

### AD5: 菜单流程

```
MainMenu
├── [单人模式]     → 直接进入 Playing（当前流程，后续 #4 改为房间配置）
└── [局域网模式]   → LanLobby (房间列表页)
                      ├── [创建房间] → 选地图+人数 → 启动 RelayRuntime → 广播 Beacon → 等待加入
                      └── [加入房间] → Connect → JoinRoom → Lobby → GameStarted
```

新增 `GameState::LanLobby` 作为局域网大厅页面。

## 拆分子任务

| Issue | 标题 | 说明 |
|-------|------|------|
| #3 | Epic: 局域网房间列表 | 跟踪整体进度 |
| #5 | 发现协议扩展 + 房间元数据 | 扩展 LanDiscoveryPacket，room_id、protocol_version 等 |
| #6 | LocalSessionHost 生命周期 | 创建房间时启动/停止 RelayRuntime，广播 Beacon |
| #7 | 房间列表 UI（LanLobby 页面） | 动态刷新房间列表，创建/加入按钮 |
| #8 | 加入流程 + Relay 分配 player_id | JoinRoom 协议，slot 分配，RoomSnapshot 返回 |
| #9 | 菜单拆分 | 单人模式/局域网模式独立入口，GameState::LanLobby |
| #1 | [阻塞] P1 命令不执行 | 必须先修复否则联机开了不能玩 |
| #2 | [阻塞] HUD 跨玩家影响 | 同上 |

## Risks / Trade-offs

- **[R1] 房主掉线全局结束** → MVP 接受。后续加 Host Migration 或 Dedicated Relay。
- **[R2] UDP 广播不跨子网** → LAN 的天然限制，可接受。跨子网需要公网 Lobby。
- **[R3] 多房主端口随机冲突** → OS 分配 (`:0`) 极小概率碰撞，不影响功能。
- **[R4] 现有 relay 单 session 限制** → 每个房主一个 relay 进程，无共享状态。

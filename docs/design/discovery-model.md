## Context

当前 `LanDiscoveryPacket` 只有 `magic`/`version`/`relay_port`，无法支持房间列表展示。更根本的问题是：**房间发现模型和传输层耦合**——`relay_port` 直接嵌入数据包，未来公网大厅无法复用同一套房间数据。

详见 #3 EPIC 设计文档 `docs/design/lan-room-list.md`（不变量 I1-I4）。

## Goals / Non-Goals

**Goals：**
- 定义 `RoomMetadata`：跨网络无关的房间领域模型
- 定义 `RoomAdvertisement`：`RoomMetadata` + 连接信息
- 重构 `LanDiscoveryPacket` 为传输层信封
- 将发现协议独立为 `bevy_adapter::discovery` 模块
- 现有 `LanServers`/`LanDiscoveryListener` 功能保留但适配新模型

**Non-Goals：**
- 不改 relay 内部 tick/命令协议（数据面）
- 不改 `LanDiscoveryPacket` 序列化格式（当前 bincode 兼容扩展）
- 不实现公网 HTTP Lobby（仅预留模型兼容）

## 数据模型

### RoomMetadata（纯房间领域模型，零网络信息）

```rust
// bevy_adapter::discovery::model

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomId(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RoomState {
    Waiting,
    Starting,
    Playing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomMetadata {
    pub room_id: RoomId,
    pub room_name: String,
    pub map_id: String,
    pub current_players: u8,
    pub max_players: u8,
    pub state: RoomState,
}
```

**不变式：** `RoomMetadata` 不能包含 IP、端口、Relay、TCP/UDP/WS 等传输层概念。

### RoomAdvertisement（发现层：房间 + 连接方式）

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelayId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomAdvertisement {
    pub relay_id: RelayId,       // relay 实例标识（区分重启）
    pub endpoint: SocketAddr,    // "ip:port"
    pub room: RoomMetadata,
}
```

### LanDiscoveryPacket（UDP 传输信封）

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LanDiscoveryPacket {
    pub magic: u16,         // 0xBEEF
    pub version: u16,       // Discovery Protocol Version（当前=1）
    pub advertisement: RoomAdvertisement,
}
```

## 模块结构

```
bevy_adapter/
├── discovery/
│   ├── mod.rs          → 重新导出
│   ├── model.rs        → RoomId, RelayId, RoomMetadata, RoomState, RoomAdvertisement
│   ├── packet.rs       → LanDiscoveryPacket（序列化/反序列化）
│   ├── listener.rs     → LanDiscoveryListener（现有，适配新模型）
│   └── broadcaster.rs  → UDP 广播逻辑（从 relay 抽取）
├── network.rs          → 去掉 LanDiscoveryPacket，保留 RelayServer 等
└── ...
```

## 兼容性

现有 `LanDiscoveryPacket` 序列化格式变化：
- `room: RoomMetadata` 替代旧字段
- `advertisement` 结构体内嵌
- 客户端发现版本不匹配时静默忽略（不 crash）

## 影响范围

| 文件 | 改动 |
|------|------|
| `bevy_adapter/src/network.rs` | 移除 `LanDiscoveryPacket`，改为引用 `discovery` 模块 |
| `bevy_adapter/src/discovery/model.rs` | **新增** |
| `bevy_adapter/src/discovery/packet.rs` | **新增** |
| `bevy_adapter/src/discovery/listener.rs` | **新增**（从 `bevy_adapter::lan` 迁移） |
| `bevy_adapter/src/discovery/broadcaster.rs` | **新增**（从 `relay/src/lib.rs` 迁移） |
| `bevy_adapter/src/lib.rs` | 注册 `discovery` 模块 |
| `render_view/src/ui/lan.rs` | 改用 `RoomAdvertisement` + `RoomMetadata` |
| `relay/src/lib.rs` | UDP 广播改用新模型 |
| `relay/Cargo.toml` | 可能新增 `uuid` 依赖 |

## 依赖

| Issue | 关系 |
|-------|------|
| #9 | ✅ 已完成，LanLobby 已就绪 |
| #6 | #5 定义模型 → #6 使用模型（依赖反转） |
| #10 | 房间名自定义（未来，模型已预留 `room_name: String`） |

## Risks / Trade-offs

- **[R1] 重构现有序列化格式** → `LanDiscoveryPacket` 当前被 relay 和 client 两端使用，需同步更改，不存在向前兼容问题（所有实例同时更新）
- **[R2] `uuid` 新依赖** → 只加在 `bevy_adapter`/`relay`，不会污染 `simulation`
- **[R3] 抽象过早？** → 不。`RoomMetadata` 无网络信息这个约束正是为了让 LAN 和公网大厅复用同一套模型

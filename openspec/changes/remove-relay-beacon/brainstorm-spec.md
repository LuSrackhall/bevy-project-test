## Context

Change A 在 `ThreadRelayRuntime` 中实现了完整的 UDP beacon 广播（RoomMetadata 驱动、双目标广播、error tolerant）。Change B1 抽取共享 relay 运行时后，`relay` crate 成为 `relay_core` 的薄包装。

但 `crates/relay/src/lib.rs` 中残留了一份独立的 UDP beacon 实现（第 30-59 行），使用硬编码的默认值（`current_players: 0`、`map_id: "grassland_small"`、单目标广播），且无停止信号。该 beacon 在各方面均劣于 `ThreadRelayRuntime` 的权威版本。

## Goals / Non-Goals

**Goals：**
- 删除 `crates/relay/src/lib.rs` 中冗余的 UDP beacon 代码（30-59 行）
- 清理不再使用的 imports（`UdpSocket`、`LanDiscoveryPacket`、`RoomAdvertisement`、`RoomId`、`RoomMetadata`、`RoomState`、`Duration`）
- 清理不再需要的 TODO 注释（"Change B2: Remove redundant beacon"）

**Non-Goals：**
- 修改 Cargo.toml（tokio 的 `time` feature 仍被传递依赖引入，移除无实际收益）
- Cargo.toml 中 `bincode` 依赖清理（留待后续观察）

## Decisions

删除即可。无架构决策。relay crate 的 TCP accept + handle 功能完全不受影响，因为 `run_relay` 不依赖 beacon。

## Risks / Trade-offs

| 风险 | 影响 | 缓解 |
|---|---|---|
| relay CLI 不再广播 UDP 信标 | 手动启动 relay CLI 时其他客户端无法自动发现 | CLI 是调试/CI 工具，联机使用 ThreadRelayRuntime 内嵌版本 |

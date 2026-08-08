## Why

Change 1-3 交付了规模、正确性、可靠 UDP、重连分页,但重连恢复只支持 **Scene A(网络断开,进程存活)**。**Scene B(进程重启)不可恢复**:全新进程(last_tick_consumed=0)不请求日志、relay 不重发 GameStarted、无世界重建路径。进程崩溃/重启是真实场景——本 change 让重启后的客户端重建世界、快速回放全量日志、续接实时对局。复用 Change 3 分页(元数据已携带 seed/players,需补 map_size)。

## What Changes

- **无条件 ReconnectRequest + 有界重试**:客户端总是发送 ReconnectRequest(last=0 也发);JoinRejected/Error 不再按 `last_tick_consumed()==0` 放弃,改为有界重试(桥接 relay 1.5s 心跳窗口)。**C1 修复**。
- **relay 补发 GameStarted**:进行中对局下,复用 Disconnected 席位的重连者收到 GameJoined 后补发 GameStarted(含 seed)。**C2 修复**。
- **Scene B 重建路径**:lobby 系统新增 ReconnectResponse 分支,用重连元数据(seed+map_size)设置 NetworkGameStart → 转 Playing → reset_game_system 用 `rebuild_world` + `generate_map` 建世界 → 页面逐页灌入 → driver 回放。
- **Scene A/B 判定**:按 driver 生命周期状态(Playing vs Lobby)区分,Playing 断在 tick0 仍走 Scene A 不重建。
- **BREAKING(协议)**:`ReconnectResponse` 增 `map_size` 字段(消除 reset_game_system 硬编码 Medium 的 desync 隐患)。
- **快速回放追赶**:Scene B 重建后批量 run_tick(每帧 N tick)追平,避免 20Hz 逐 tick 数分钟。
- **回放期上发门控**:回放追赶期间 network_flush_system 跳过,防已定稿 tick 帧泄漏到 relay staging。

无 simulation 改动;Scene A 路径不回退;确定性保持(同 init + run_tick(enable_ai:false))。

## Capabilities

### New Capabilities

(无——Scene B 是 network-reconnect 的能力补全)

### Modified Capabilities
- `network-reconnect`: Scene B 进程重启重建实现(无条件重连请求 + 有界重试 + 重建/快速回放 + map_size 字段 + A/B 判定)
- `relay-server`: 进行中对局下复用 Disconnected 席位的重连者补发 GameStarted

## Impact

- **network.rs**:`ReconnectResponse` 增 `map_size`;handle_reconnect 携带;Scene A/B 判定辅助
- **relay_core.rs**:JoinGame 复用 Disconnected 席位且 game_started 时补发 GameStarted
- **transport.rs**:无条件 ReconnectRequest;JoinRejected/Error 有界重试;回放期上发门控
- **driver.rs**:回放追赶模式(批量 run_tick)
- **render_view/src/lib.rs**:lobby 系统 ReconnectResponse 分支 → NetworkGameStart + 转 Playing;reset_game_system 用 map_size
- **session/reconnect.rs**:`rebuild_world` 扩展(map_size 参数)
- **测试**:handle_reconnect(last=0)+map_size 单测、relay 补发 GameStarted 单测、A/B 判定边界单测、进程重启重建+快速回放 hash 全等(MoveTo 真实命令 + 确定性步进)、Scene A 回归、快速重启重试桥接
- **文档**:specs/network-reconnect Scene B 更新为已实现;ADR-0009 补记 GameStarted 重发

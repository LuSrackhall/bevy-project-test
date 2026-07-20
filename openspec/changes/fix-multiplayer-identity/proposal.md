## Why

#1（P1 命令不执行）和 #2（HUD 跨玩家影响）的根因都是身份模型未正确落地。连接加入方使用 `player_id = 0`（占位值），且 `consume_commands_system` 不检查命令的 `player_id` 是否匹配目标单位的归属阵营。

## What Changes

**Fix A：GameJoined 更新 NetworkCommandSource**
- `GameJoined` 事件通过 `NetworkEventReceiver` 跨线程传递
- Bevy 主线程读取事件后更新 `SimulationDriver.source.player_id`
- `LocalPlayerIdentity` Resource 同步更新

**Fix B：Simulation Validation Stage**
- 在 `run_tick()` 中新增 `validate_commands()` 函数
- 规则：玩家只能操作自己阵营的单位
- `consume_commands_system` 职责不变（只执行，不验证）

**ADR：Command Envelope 架构说明**

## Capabilities

### New Capabilities
- `simulation-validation`: Simulation Validation Stage + validate_commands()
- `identity-pipeline`: GameJoined → NetworkCommandSource.player_id 更新

### Modified Capabilities
<!-- 无现有 spec 变更 -->

## Impact

- `simulation/src/lib.rs`：新增 `validate_commands()` + 在 `run_tick()` 中调用
- `render_view/src/lib.rs`：`lobby_update_system` 处理 `NetworkEvent::GameJoined`
- `bevy_adapter/src/network.rs`：`NetworkCommandSource` 的 `player_id` 字段设为 `pub`
- `docs/adr/`: 新增 Command Envelope ADR

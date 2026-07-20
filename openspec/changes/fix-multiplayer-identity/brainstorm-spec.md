## Context

前 5 个子任务（#5-#9）已完成局域网房间列表和加入流程的基础设施，但 #1 和 #2 两个阻塞 Bug 尚未修复：

- **Bug #1（P1 命令不执行）**：加入方发出的右键移动/攻击命令不生效
- **Bug #2（HUD 跨玩家影响）**：房主的 HUD 操作（如切换出兵类型）影响加入方单位

#8（加入流程）建立了 Relay-authoritative player identity，为修复这两个 Bug 提供了身份基础。

## Goals / Non-Goals

**Goals：**
- Fix A：`GameJoined` 事件跨线程更新 `NetworkCommandSource.player_id`，使加入方使用 Relay 分配的 ID
- Fix B：在 `run_tick()` 中增加 Simulation Validation Stage，过滤玩家操作非所属阵营单位的命令
- ADR：记录 `GameCommand.player_id` 属于 Envelope 数据，未来可抽
- 单人模式不受影响，AI 命令不受影响，Replay 不受影响

**Non-Goals：**
- 不重构成 Command Envelope/Payload 分离
- 不修改 Replay 序列化格式
- 不修改 `consume_commands_system` 的职责

## Decisions

### AD1: Fix A — GameJoined 事件更新 NetworkCommandSource

```
tokio 线程收到 GameJoined { player_id }
  → NetworkEventReceiver.push(NetworkEvent::GameJoined { player_id })
  → Bevy 主线程 lobby_update_system 读取该事件
  → SimulationDriver.source (NetworkCommandSource).player_id = assigned_player_id
  → LocalPlayerIdentity Resource 更新
```

复用现有的 `NetworkEvent::GameJoined` 变体，不新增事件类型。`GameJoined` 本身就是 Relay 分配身份的权威语义。

### AD2: Fix B — Simulation Validation Stage

在 `run_tick()` 中的命令排序之后、`consume_commands_system` 之前，插入验证阶段：

```rust
// simulation/src/lib.rs
// ── Step 2.5: Simulation Validation ──
// Rejects commands that violate simulation integrity rules.
// This is a Simulation Validation Boundary, not a security boundary.
// Current rules:
//   1. Player can only command units of their own faction (fixes #1, #2).
commands = validate_commands(world, commands, &known_players);
```

`consume_commands_system` 保持专注于命令执行，不承担验证职责。

未来 Replay、AI、网络等所有命令来源统一经过该验证阶段。

### AD3: 单人模式和 AI 兼容性

- 单人模式：`PlayerSlots` 可能不存在，`slot_id == player_id == faction`，验证一律通过
- AI 命令：AI 的 `player_id` 对应 `PlayerSlots` 中 AI slot 的 `FactionId`，验证通过
- Replay：记录的已通过验证的命令，回放时再次通过验证（结果应一致）

### AD4: Architecture Note — Command Envelope

`GameCommand` 当前混入 `player_id`（Envelope 字段）和 `action`（Payload 字段）。未来如果拆分为 `AuthenticatedCommand { player_id, tick, GameCommand }`，职责会更清晰。当前不修改。

## Risks / Trade-offs

- **[R1] Fix A 跨线程更新**：NetworkEventReceiver 已有完整的事件传递机制，风险低
- **[R2] Fix B 性能**：`validate_commands` 对每 tick 每命令做一次 entity lookup，复杂度 O(n * m)，对于极端场景（数百单位同时命令）有轻微性能影响。可接受。
- **[R3] 多人模式下 replay 一致性**：验证通过的命令在 replay 时再次通过，确定性不变。

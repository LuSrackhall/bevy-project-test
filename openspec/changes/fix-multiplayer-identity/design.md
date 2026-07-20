## Context

详见 `brainstorm-spec.md`（AD1-AD4 设计决策）。#1 和 #2 的根因是身份模型未正确落地。

## Goals / Non-Goals

**Goals:**
- Fix A：GameJoined 跨线程更新 NetworkCommandSource
- Fix B：run_tick 中 validate_commands（阵营归属检查）
- ADR：Command Envelope 架构说明

**Non-Goals:**
- 不修改 consume_commands_system 职责
- 不拆分 Command Envelope

## Decisions

### Fix A 实现

在 `transport.rs` 的 `GameJoined` 处理器中，推送 `NetworkEvent::GameJoined { player_id, player_count }`（新增变体）。在 `render_view/src/lib.rs` 的 `lobby_update_system` 中读取该事件，更新 `SimulationDriver.source`：

```rust
if let NetworkEvent::GameJoined { player_id, .. } = event {
    if let Some(ref mut d) = _driver {
        if let CommandSource::Network(ref mut ns) = d.source {
            ns.player_id = player_id;
        }
    }
}
```

### Fix B 实现

```rust
// simulation/src/lib.rs
fn validate_commands(world: &World, commands: Vec<GameCommand>, known_players: &[u8]) -> Vec<GameCommand> {
    let slots = world.get_resource::<PlayerSlots>();
    commands.into_iter().filter(|cmd| {
        // NoOp always passes (placeholder, no target)
        if matches!(cmd.action, Action::NoOp) { return true; }
        // Map player_id → FactionId
        let faction = slots.and_then(|s| s.slots.iter()
            .find(|slot| slot.slot_id.0 == cmd.player_id)
            .map(|slot| slot.faction))
            .unwrap_or(FactionId(cmd.player_id));
        // Find target entity and check faction
        // ... (uses existing UnitIdEntityIndex + FactionComponent query)
    }).collect()
}
```

### Module Changes

- `simulation/src/lib.rs` — 新增 `validate_commands` + `run_tick` 中调用
- `render_view/src/lib.rs` — `lobby_update_system` 处理 `GameJoined` 事件
- `docs/adr/0007-command-envelope.md` — 架构说明

## Risks

- **[R1]** Fix B 的 entity lookup 性能。使用 `UnitIdEntityIndex`（O(1) lookup）。
- **[R2]** Fix A 跨线程事件可能存在竞争：`GameJoined` 在 `NetworkCommandSource` 创建前到达。通过检查 `source` 是否为 `Network` 类型确保安全。

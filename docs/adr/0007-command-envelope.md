# ADR 0007: Command Envelope 与 Payload 分离（Architecture Note）

## 状态

**Date**: 2026-07-20
**Status**: Accepted (Architecture Note — not implemented)

## 背景

`GameCommand` 当前结构同时包含 Envelope 字段和 Payload 字段：

```rust
pub struct GameCommand {
    pub tick: u32,          // Envelope — 由 NetworkCommandSource/Driver 填充
    pub player_id: u8,      // Envelope — 由 Relay 分配（#8 Identity Pipeline）
    pub action: Action,     // Payload — 真正的命令内容
}
```

`player_id` 和 `tick` 本质上是命令的传输上下文（Envelope），不是命令本身的内容（Payload）。但当它们被序列化到 Replay 文件时，两者被混在一起存储。

## 当前不做分离的原因

1. **Replay 序列化格式**：重放时依赖 `player_id` 进行命令分配。分离后需要额外的 Replay 格式适配。
2. **排序依赖**：`run_tick` 中的命令排序按 `(player_id, sort_tag)` 进行，如果 Envelope 分离，排序逻辑也需要适配。
3. **NoOp 注入**：`run_tick` 中使用 `player_id` 填充缺失玩家的 NoOp 命令。
4. **涉及范围广**：从 relay 协议到 simulation 到 replay，全部涉及。

## 未来路径

如需拆分，建议的方案：

```rust
/// Authenticated command — Envelope
pub struct AuthenticatedCommand {
    pub tick: u32,
    pub player_id: u8,
    pub command: GameCommand,
}

/// Pure command payload — no network/metadata concerns
pub struct GameCommand {
    pub action: Action,
}
```

届时 `consume_commands_system` 将接收 `Vec<GameCommand>`（已剥离 `player_id`），`player_id` → `FactionId` 的映射验证已由 `validate_commands` 在 `run_tick` 中完成。

## 关联

- #8: Relay-authoritative player identity
- `fix-multiplayer-identity`: validate_commands（Simulation Validation Boundary）
- See `openspec/changes/fix-multiplayer-identity/brainstorm-spec.md` AD4

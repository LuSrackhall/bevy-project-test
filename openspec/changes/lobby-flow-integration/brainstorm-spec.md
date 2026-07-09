## Context

宪法审计（lifecycle-audit）和流程审计（flow-audit）确认 lobby 流程有 2 个小缺口需要收尾。flow-audit 同时排除了 3 项非必要改动。

## Goals

1. `.after(lobby_update_system)` 显式排序约束（~2行）
2. Cancel 按钮移除 3 个重复资源清理（~3行）

## Decisions

### D1：排序约束

```rust
// ui/mod.rs
lobby::update_lobby_status.after(crate::lobby_update_system).run_if(in_state(GameState::Lobby))
```

### D2：Cancel 去重

保留 4 个 transport 资源清理（NetworkSender/Receiver/EventReceiver/Handle），移除 3 个由 OnExit(Lobby) 兜底的重复项（network_active/LobbyConnectionState/ConnectionPollRx）。

## Risks

| Risk | Mitigation |
|------|-----------|
| Cancel 遗漏传输资源 | 保留 4 项 transport 清理 |
| 排序约束影响其他 | `.after()` 是增量约束 |

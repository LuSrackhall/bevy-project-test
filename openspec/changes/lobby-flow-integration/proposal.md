## Why

宪法审计发现 2 个小缺口：系统排序约束缺失 + Cancel 按钮重复清理。需要收尾加固 lobby 流程。

## What Changes

1. `.after(lobby_update_system)` 排序约束
2. Cancel 按钮移除 3 个重复资源清理

## Impact

- `crates/render_view/src/ui/mod.rs` — 添加 `.after()` 排序
- `crates/render_view/src/ui/lobby.rs` — Cancel 按钮去重

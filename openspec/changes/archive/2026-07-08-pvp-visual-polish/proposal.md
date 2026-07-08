## Why

联机 Player 2 在 debug 模式下看到的己方单位显示为红色（敌色），造成视觉误导。

## What Changes

4 处 `FactionId(0/1/2)` 颜色映射改为动态 `FactionId(lid)` 分派

## Capabilities

### New Capabilities

- `pvp-debug-colors`: 调试可视化颜色基于 LocalPlayerId 动态分派

## Impact

- `crates/render_view/src/debug_shape.rs` — 4 处 match → if-else + 2 个辅助函数

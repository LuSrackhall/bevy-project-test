## Why

联机模式下 HUD 显示层（顶部栏城市人口、寻敌面板计数）硬编码 `FactionId(0)` 过滤数据，导致 Player 2 看到的是 Player 1 的统计数据而非自身的。上一变更 pvp-hud-command-fix 已修复命令归属，但显示层数据仍不正确。

## What Changes

1. 4 处 `FactionId(0)` 硬编码 → `FactionId(lid)`，其中 `lid = crate::local_player_id(&*sim_world)`
2. Match arm 标签简化：guard 匹配本地玩家，统一 fallback 到 "其他"

## Capabilities

### New Capabilities

- `hud-display-player-id`: HUD 显示层（人口统计、兵种计数、阵营标签）使用 `LocalPlayerId` 动态确定当前玩家。

### Modified Capabilities

*无。*

## Impact

- `crates/render_view/src/ui/hud.rs` — 4 处 FactionId 过滤 + 1 处 match arm
- `crates/render_view/src/lib.rs` — 复用已存在的 `local_player_id()` 公共函数
- 单机兼容：✅ `lid=0` 行为不变

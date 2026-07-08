## Why

联机模式下胜负判定系统硬编码 `FactionId(0)→has_player`、`FactionId(1)→has_enemy`，导致 Player 2（`LocalPlayerId(1)`）的胜负判定错误：Player 2 消灭 Player 1 后游戏异常结束。

三个子 Agent 审计同时发现 `else { has_enemy = true }` 会误将 FactionId(2) 中立城市标记为 enemy，导致游戏中立城市未全部占领时永不结束。

## What Changes

`check_victory_system` 重写：使用 `LocalPlayerId` 确定己方阵营 + `PlayerSlots` 过滤活跃玩家阵营（忽略中立 FactionId(2)）。

## Capabilities

### New Capabilities

- `pvp-victory-sync`: 胜负判定使用 `LocalPlayerId` + `PlayerSlots` 动态确定对阵双方，忽略中立阵营。

## Impact

- `crates/render_view/src/lib.rs` — `check_victory_system` 约 25 行重写
- 宪法合规：✅
- 单机兼容：✅
- 联机同步：lockstep 自动同步

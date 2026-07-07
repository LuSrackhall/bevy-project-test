## Why

联机模式下 HUD 按钮发出的 `GameCommand` 硬编码 `player_id: 0`（Player 1 阵营），导致 Player 2 的按钮指令被路由到 Player 1 的单位而非自己的单位。Player 2 的实际命令归属错误，PvP 场景下无法控制己方单位。

`selection.rs` 和 `camera.rs` 已有正确的 `local_player_id()` 模式（通过 `SimulationWorld.world_ref().get_resource::<LocalPlayerId>()` 动态读取），但 HUD 按钮未采用此模式。

此外，`session.rs` 的 `resolve_intent()` / `GameIntent` / `GameKind` 类型是定义后从未被调用的死代码，持续产生误导。

## What Changes

1. **HUD 按钮 player_id 修复** — 4 处硬编码 `player_id: 0` → 从 `SimulationWorld` 中的 `LocalPlayerId` 资源动态读取
2. **提取公共辅助函数** — `local_player_id()` 从 `selection.rs` 提到 `render_view/src/lib.rs` 作为 `pub(crate)` 函数，迁移 `camera.rs` 的内联实现
3. **删除死代码** — 移除 `render_view/src/session.rs`（`resolve_intent()` / `GameIntent` / `GameKind`）及对应的 `pub mod session`
4. **回归测试** — 验证 `LocalPlayerId` 在默认不存在时回退为 0

## Capabilities

### New Capabilities

- `hud-player-id`: HUD 按钮命令的玩家 ID 从仿真世界的 `LocalPlayerId` 资源动态读取，而非硬编码。包含从 `selection.rs` 提取的公共辅助函数。

### Modified Capabilities

*无。所有修改在 render_view 层内，不改变任何现有 spec 的 REQUIREMENTS。*

## Impact

- `crates/render_view/src/ui/hud.rs` — 4 行硬编码 player_id 改动
- `crates/render_view/src/lib.rs` — 新增 `pub(crate) fn local_player_id()`，移除 `pub mod session`
- `crates/render_view/src/selection.rs` — `local_player_id()` 改为调用公共函数
- `crates/render_view/src/camera.rs` — 内联读取改为调用公共函数
- `crates/render_view/src/session.rs` — **删除**
- `crates/simulation/src/command.rs` — 新增回退测试
- 宪法合规：✅ 全部通过（§1.2、§2.5、§11、§17）
- 单机兼容：🟢 无变化（`unwrap_or(0)` 回退 0）
- 测试现状：137 测试全部通过，保持不变

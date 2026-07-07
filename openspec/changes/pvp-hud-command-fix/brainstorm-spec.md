## Context

当前 `render_view/src/ui/hud.rs` 中 4 个 HUD 按钮观察者（observer closure）硬编码 `player_id: 0` 作为 GameCommand 的玩家 ID。这导致联机模式下 Player 2（LocalPlayerId ≠ 0）的按钮指令错误地归属到 Player 1 的阵营。

已有正确先例：
- `render_view/src/selection.rs:13-18` 定义了 `local_player_id()` 辅助函数，通过 `sim.world_ref().get_resource::<LocalPlayerId>().map(|r| r.0).unwrap_or(0)` 读取本地玩家 ID
- `render_view/src/camera.rs:27-29` 使用相同模式

此外，`render_view/src/session.rs` 的 `resolve_intent()`、`GameIntent`、`GameKind` 类型是死代码——被定义但从未被生产路径调用，实际网络流程直接走 `NeedsGameReset::Network`。

### 相关文件

| 文件 | 当前状态 |
|------|----------|
| `crates/render_view/src/ui/hud.rs` | 4 处 `player_id: 0` 硬编码 |
| `crates/render_view/src/session.rs` | 死代码 `resolve_intent()` |
| `crates/render_view/src/lib.rs` | 需添加公共辅助函数 + 删除 `pub mod session` |
| `crates/render_view/src/selection.rs` | 已有 `local_player_id()`（将提取到公共位置） |
| `crates/render_view/src/camera.rs` | 已有内联 `LocalPlayerId` 读取（将迁移到公共函数） |

## Goals / Non-Goals

### Goals

- HUD 中 4 处 `player_id: 0` → 动态读取 `LocalPlayerId` 资源
- 提取 `local_player_id()` 公共辅助函数到 `render_view/src/lib.rs`
- 将 `selection.rs` 和 `camera.rs` 的重复实现迁移到公共函数
- 删除死代码文件 `session.rs`
- 添加最小回归测试验证 `LocalPlayerId` 回退语义
- 单机模式行为不变（`unwrap_or(0)` 回退 0，与硬编码一致）
- 宪法合规（§1.2 依赖禁令、§2.5 命令流水线、§17 真相归属）

### Non-Goals

- 不改双玩家集成测试（`network_e2e.rs`）
- 不改 observer 闭包架构
- 不改仿真层行为
- 不做 HUD 按钮的 Bevy 集成测试（闭包无法直接测试）
- 不在 `init_simulation_world` 中添加 `init_resource<LocalPlayerId>`（可作为后续优化）

## Decisions

### D1：通过 `NonSend<SimulationWorld>` 读取 LocalPlayerId

`LocalPlayerId` 存在于 simulation ECS World 中，不是主 Bevy World。在 HUD observer 闭包中需要通过 `NonSend<SimulationWorld>` 或 `NonSendMut<SimulationWorld>` 参数访问，然后用 `sim.world_ref().get_resource::<LocalPlayerId>()` 读取。

### D2：按需添加参数

| 闭包 | 已有 SimWorld 参数 | 需要新增 |
|------|-------------------|---------|
| SpawnTypeBtn (line 287) | 无 | `NonSend<SimulationWorld>`（参数 6→7） |
| ShieldButton (line 367) | `NonSendMut<SimulationWorld>` | 无 |
| SeekIssueBtn (line 500) | `NonSendMut<SimulationWorld>` | 无 |

### D3：提取公共辅助函数

将 `local_player_id()` 从 `selection.rs` 提取到 `render_view/src/lib.rs` 作为 `pub(crate)` 函数，并迁移 `camera.rs` 的内联实现。

### D4：删除 session.rs

`resolve_intent()`、`GameIntent`、`GameKind` 未被任何生产代码引用。删除文件 + 移除 `lib.rs` 中的 `pub mod session`。

### D5：单机回退保证

所有读取点使用 `map().unwrap_or(0)` 模式：当 `LocalPlayerId` 资源不存在时（单机模式），回退值为 0，与当前硬编码行为完全一致。

## Risks / Trade-offs

| Risk | 评级 | Mitigation |
|------|------|-----------|
| 单机默认值回退 | 🟢 极低 | `unwrap_or(0)` 行为与 `player_id: 0` 硬编码相同 |
| Observer param 超限 | 🟢 极低 | 最大 8 param，Bevy 上限 16 |
| 写入仿真世界 | 🟢 无 | 只读不写，§17 合规 |
| session.rs 删除遗漏引用 | 🟢 极低 | 已通过 grep 确认 0 外部引用 |
| 回放模式误触 | 🟢 无 | 4 个闭包首行均有 `if Replay { return; }` 守卫 |

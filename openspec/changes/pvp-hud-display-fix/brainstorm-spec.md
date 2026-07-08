## Context

`test-edge-cases` agent 在 pvp-hud-command-fix 的审计中发现 `render_view/src/ui/hud.rs` 中有 4 处显示层 `FactionId(0)` 硬编码：

- `update_top_bar` 中 2 处（城市人口过滤 + 阵营标签）
- `seek_panel_count_system` 中 2 处（选中单位过滤 + 全局单位过滤）

当前 PvP 联机模式下 Player 2（`LocalPlayerId(1)`）可以看到正确的命令归属（受益于上一变更 pvp-hud-command-fix），但 HUD 面板和寻敌面板统计仍然显示 Player 1（`FactionId(0)`）的数据。

已在 `render_view/src/lib.rs` 中存在 `pub(crate) fn local_player_id(&SimulationWorld) -> u8`，`update_top_bar` 和 `seek_panel_count_system` 都已持有 `sim_world: NonSend<SimulationWorld>` 参数，无需新增参数。

### 相关文件

| 文件 | 行号 | 当前状态 |
|------|------|----------|
| `crates/render_view/src/ui/hud.rs` | 625 | `f.0 == simulation::types::FactionId(0)` |
| `crates/render_view/src/ui/hud.rs` | 656 | `FactionId(0) => "玩家"` |
| `crates/render_view/src/ui/hud.rs` | 1181 | `fac.0 == FactionId(0)` |
| `crates/render_view/src/ui/hud.rs` | 1208 | `fac.0 == FactionId(0)` |

## Goals / Non-Goals

**Goals:**
- 4 处 `FactionId(0)` 硬编码 → `FactionId(lid)`（lid 通过 `crate::local_player_id()` 获取）
- match arm 标签改为 guard 匹配，简化未覆盖分支
- 单机模式行为不变（`lid=0`，与改前一致）
- 宪法合规（§1.2 依赖禁令、§17 真相归属）

**Non-Goals:**
- 不改 `render_view/src/lib.rs:199` 的 `check_victory_system`（作为独立变更）
- 不改 `debug_shape.rs` 的 faction 颜色（YAGNI）
- 不改非 hud.rs 的 FactionId 引用

## Decisions

### D1：4 处 FactionId(0) → FactionId(lid)

| 行号 | 函数 | 当前 | 修复 |
|------|------|------|------|
| 625 | update_top_bar | `f.0 == FactionId(0)` | `f.0 == FactionId(lid)` |
| 656 | update_top_bar | `FactionId(0) => "玩家"` | `f if *f == FactionId(lid) => "玩家"` |
| 1181 | seek_panel_count_system | `fac.0 == FactionId(0)` | `fac.0 == FactionId(lid)` |
| 1208 | seek_panel_count_system | `fac.0 == FactionId(0)` | `fac.0 == FactionId(lid)` |

### D2：Match arm 简化

```rust
// 改前
FactionId(0) => "玩家",
FactionId(1) => "敌人",
FactionId(2) => "中立",
FactionId(_) => "其他",

// 改后
f if *f == FactionId(lid) => "玩家",
_ => "其他",
```

FactionId(1) → "敌人" 和 FactionId(2) → "中立" 在 `lid ≠ 0` 时会产生错误标签，因此统一收敛到 "其他"，直到未来引入完整的 faction→label 映射时再扩展。

### D3：lid 获取方式

在两个函数体内添加 `let lid = crate::local_player_id(&*sim_world);`。`NonSend<SimulationWorld>` 已存在，零参数成本。

## Risks / Trade-offs

| Risk | 评级 | Mitigation |
|------|------|-----------|
| 单机 lid=0 与改前一致 | 🟢 无 | `local_player_id()` 回退 0 |
| Match arm 合并后丢失阵营标记 | 🟢 低 | 当前架构只有"本地玩家/其他"概念，简化标签正确 |
| 遗漏其他 FactionId(0) | 🟢 无 | 已审计 hud.rs 内外共 7 处引用，确认 4 处需改 |

## Post-Implementation Confirmation

### Confirmed: check_victory_system (Scope外)

三个子Agent 一致确认 `render_view/src/lib.rs:199` 的 `check_victory_system` 中 `FactionId(0)`/`FactionId(1)` 硬编码会导致联机模式下 Player 2 的胜负判定异常。此问题不属于本次变更范围，需独立变更修复。

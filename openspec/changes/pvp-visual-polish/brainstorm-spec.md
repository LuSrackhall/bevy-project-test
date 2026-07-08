## Context

`crates/render_view/src/debug_shape.rs` 中 4 处 `FactionId(0/1/2)` 颜色映射硬编码，导致联机 Player 2（控制 `FactionId(1)`）的调试颜色显示错误：己方单位显示为红色（敌色）而非蓝色（己色）。

`draw_debug_shapes_system` 已持有 `sim_world: NonSend<SimulationWorld>` 参数，可直接调用 `crate::local_player_id()`。

## Goals / Non-Goals

**Goals:**
- 4 处颜色映射改为基于 `LocalPlayerId` 动态分派
- 己方→蓝色，敌方→红色，中立/其他→灰色
- 单机兼容：`lid=0` 行为不变
- 保留各实体类型（城市/士兵/朝向线/盾牌）不同的颜色值

**Non-Goals:**
- 不改非 `debug_shape.rs` 的颜色代码
- 不改 release build 行为（`#[cfg(feature)]` 门控已存在）

## Decisions

### D1：提取两个辅助函数

```rust
fn is_player_faction(f: FactionId, lid: u8) -> bool {
    f == FactionId(lid)
}
fn faction_is_active_enemy(f: FactionId, lid: u8) -> bool {
    f != FactionId(lid) && (f == FactionId(0) || f == FactionId(1))
}
```

### D2：每处 match 改为 if-else

保留各实体类型的独立颜色值，仅分派逻辑改为动态：

```rust
// 改前
FactionId(0) => city_blue,
FactionId(1) => city_red,
FactionId(2) | FactionId(_) => city_gray,

// 改后
if is_player_faction(faction, lid) => city_blue
else if faction_is_active_enemy(faction, lid) => city_red
else => city_gray
```

### D3：lid 获取

在 `draw_debug_shapes_system` 顶部添加 `let lid = crate::local_player_id(&*sim_world);`。

## Risks

| Risk | 评级 | Mitigation |
|------|------|-----------|
| 单机兼容 | 🟢 无 | `lid=0`，行为不变 |
| `#[cfg]` 门控 | 🟢 无 | debug_render 仅在 debug 模式下编译 |

## Post-Implementation Confirmation

两个子Agent 确认：
1. 12 种颜色分派组合中所有实际可达场景正确
2. 宪法合规
3. 单机兼容
4. cfg 门控正常

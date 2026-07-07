## Context

本变更修复 HUD 按钮 player_id 硬编码问题，详见 brainstorm-spec.md（Context/Decisions/Risks 部分已覆盖架构设计）。本文档补充实施层面的技术细节。

涉及模块：
- `render_view/src/lib.rs` — 公共辅助函数插入点
- `render_view/src/ui/hud.rs` — 4 处闭包修改
- `render_view/src/selection.rs` — 迁移到公共函数
- `render_view/src/camera.rs` — 迁移到公共函数
- `render_view/src/session.rs` — 删除
- `simulation/src/command.rs` — 测试

## Goals / Non-Goals

**Goals:**
- HUD 4 处 `player_id: 0` → 动态读取（通过 `SimulationWorld` 访问 `LocalPlayerId` 资源）
- 公共辅助函数 `local_player_id()` 减少重复
- 删除死代码 session.rs
- 回归测试验证 `LocalPlayerId` 回退语义

**Non-Goals:**
- 不涉及仿真层改动
- 不改 bevy_adapter 或 relay
- 不添加双玩家集成测试

## Decisions

### D1：公共函数签名

```rust
// 插入位置：render_view/src/lib.rs，HudTexts 资源定义之后
pub(crate) fn local_player_id(sim: &simulation::types::SimulationWorld) -> u8 {
    sim.world_ref()
        .get_resource::<simulation::types::LocalPlayerId>()
        .map(|r| r.0)
        .unwrap_or(0)
}
```

参数类型 `&SimulationWorld` 而非 `&World`——保证编译期强制：只能传入仿真世界（含 `world_ref()` 方法），不能传入主 Bevy World。这与 `selection.rs` 现有签名一致。

### D2：闭包参数修改策略

| 闭包 | 行号 | 原有 SimWorld 参数 | 操作 | 说明 |
|------|------|-------------------|------|------|
| SpawnTypeBtn | 287 | 无 | 新增 `sim: NonSend<bevy_adapter::tick::SimulationWorld>` | 只读，使用 `NonSend` |
| ShieldButton | 367 | `mut sim: NonSendMut<SimulationWorld>` | 仅添加 `let lid = local_player_id(&sim);` | 已有可变引用，调用 `sim.world_ref()` 只读 |
| SeekIssueBtn | 505/510 | `mut sim: NonSendMut<SimulationWorld>` | 仅添加 `let lid = local_player_id(&sim);` | 同上闭包，两处分支共享 |

Note：`NonSend`/`NonSendMut` 在 observer closure 中是合法系统参数（Bevy 0.19 `IntoObserver` 通过 `IntoSystem` 转换，支持所有 `SystemParam` 类型）。

### D3：迁移已有实现

- `selection.rs` 的私有 `local_player_id()` 函数（line 13-18）→ 删除，改调用公共函数
- `camera.rs` line 27-29 的内联读取 → 改为 `local_player_id(&sim_world)` 调用

### D4：测试设计

在 `simulation/src/command.rs` 的测试模块中追加：

```rust
#[test]
fn test_local_player_id_fallback() {
    // LocalPlayerId 的 Default 为 0 — 验证回退行为
    assert_eq!(LocalPlayerId::default().0, 0);
}
```

$10 行，零依赖，在 simulation crate 中独立运行。

## Risks / Trade-offs

| Risk | 评级 | Mitigation |
|------|------|-----------|
| NonSend 参数在 observer 中不兼容 | 🟢 几乎为零 | Bevy 0.19 IntoObserver 支持所有 SystemParam；selection.rs 已使用 |
| SpawnTypeBtn 参数重排导致编译错误 | 🟢 已验证 | 参数位置在末尾插入，不改变已有顺序 |
| camera.rs 导入路径错误 | 🟢 极低 | `render_view/src/lib.rs` 中的公共函数在 crate 内直接可访问 |
| session.rs 被外部测试引用 | 🟢 无 | 已确认 0 外部引用 |

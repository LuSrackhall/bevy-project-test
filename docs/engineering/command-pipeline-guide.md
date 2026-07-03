# Simulation Command Pipeline — 实现指南

> 关联：宪法 §2.5.4（Pipeline 固化）、ADR-006（零感知原则）
> 来源：simulation-command-pipeline Change (2026-07-02)

## 两个 CommandBuffer

系统中存在两个 `CommandBuffer`，各有职责：

| Buffer | 类型 | 角色 | 谁读 |
|--------|------|------|------|
| bevy 级 `cmd_buf` | `ResMut<CommandBuffer>` (bevy resource) | 录制源 + 命令收集 | `LiveCommandSource.commands_for_tick()` + `ReplayRecorder.record_tick()` |
| simulation 内部 | `simulation::World` 中的 `Resource<CommandBuffer>` | 执行源 | `take_for_tick()` → `consume_commands_system()` |

**数据流：**

```
observer / system
    ↓
bevy 级 cmd_buf (A)  ←──── render_view 必须推入此处
    ↓
LiveCommandSource.commands_for_tick(tick)
    ↓
ReplayRecorder.record_tick()  ── 录制到 ReplayFile
    ↓
inject_commands()  ── 转写到 simulation 内部 buffer (B)
    ↓
take_for_tick() → consume_commands_system() → simulation 执行
```

## render_view 的交互规则

render_view 对仿真层只有两种合法操作：

### 读（只读查询）

```rust
fn my_system(sim: NonSend<SimulationWorld>, ...) {
    let world = sim.world_ref();        // &simulation::World
    let mut q = sim.query::<(&ComponentA, &ComponentB)>();
    for (a, b) in q.iter(world) { ... }
}
```

### 写（推入命令）

```rust
fn my_system(mut cmd_buf: ResMut<CommandBuffer>, ...) {
    cmd_buf.push(GameCommand {
        tick: tick_clock.current_tick + 1,
        player_id: 0,
        action: Action::SomeAction { ... },
    });
}
```

### 禁止

- ❌ `NonSendMut<SimulationWorld>` — render_view 不得持有 simulation 可写引用
- ❌ `sim.world_mut()` — 仅在 bevy_adapter 内部可用（`pub(crate)`）
- ❌ `sim.submit_command()` — 推入 simulation 内部 buffer (B)，**绕过录制路径**，导致 replay DESYNC
- ❌ `cmd_buf` 之外的任何直接写入 `simulation::World` 的途径

## Architecture Test 验证的内容

`crates/bevy_adapter/tests/architecture.rs` 在 CI 中自动检查：

1. `test_render_view_no_world_mut` — render_view 源码中无 `world_mut()` 调用
2. `test_render_view_no_direct_simulation_import` — render_view 不直接导入 `use simulation::World`

## 常见错误

| 错误写法 | 后果 | 正确写法 |
|---------|------|---------|
| `sim.submit_command(cmd)` | 命令不被录制（绕过 bevy buffer A） | `cmd_buf.push(cmd)` |
| `let w = sim.world_mut()` | 编译失败（`pub(crate)`） | `let w = sim.world_ref()` |
| `NonSendMut<SimulationWorld>` | CI 架构测试警告 | `NonSend<SimulationWorld>` |

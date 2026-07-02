## Context

通过 driver 层集成测试（15000 tick, Medium map, 3 seeds）确认：**仿真层 + 命令注入 + seek 全部确定性**。DESYNC 根因不在 simulation 层。

实际根因：
1. `hud.rs` spawn type observer 直接写 `c.spawn_type = btn.0` 但不推命令 → 录制遗漏
2. `simulation_driver_system` 在 `total_ticks` 处只有注释无操作 → 回放越界 ghost ticks

## Goals / Non-Goals

见 brainstorm-spec.md

## Decisions

### D1: SpawnType 双写策略

```rust
// Immediate modification for UI feedback
if let Some(mut c) = w.entity_mut(ce).get_mut::<CityComponent>() { c.spawn_type = btn.0; }
// Push command for replay recording
cmd_buf.push(GameCommand {
    tick: tick_clock.current_tick + 1,
    player_id: 0,
    action: Action::SetSpawnType { city: cid, soldier_type: btn.0 },
});
```

直接修改 + 命令推入。Live 时用户立即看到反馈；Replay 时命令被注入，城市产出正确兵种。

### D2: Replay 边界处理

在 `simulation_driver_system` 中：

```rust
if driver.clock.current_tick >= rs.replay.total_ticks {
    driver.scheduler.is_paused = true;
}
```

在 `handle_seek` 中：cap target（`.min(max_ticks)`）防止 seek 过界。

### D3: Driver 层集成测试

三个测试覆盖确定性场景：
- `test_driver_live_replay_determinism` (15000 tick, seed 42, Small)
- `test_driver_live_replay_determinism_medium` (10000 tick, seed 99, Medium)
- `test_driver_live_replay_determinism_seed_77` (15000 tick, seed 77, Small)

## Risks / Trade-offs

- [短期双写] Live 时 observer 先直接改再推命令，后者在下个 tick 被 consume → 两次写同一字段，值相同无害
- [测试边界] 15000 tick 测试通过 ≠ 任意 replay 都通过，但提供了坚实的回归基线

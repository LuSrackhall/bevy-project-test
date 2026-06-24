## 1. SimulationDriver 核心结构

- [ ] 1.1 创建 `bevy_adapter/src/driver.rs`，定义 `TickClock`、`SchedulerState`、`CommandSource`、`LiveCommandSource`、`ReplayCommandSource`、`SimulationDriver`、`DriverContext` 结构
- [ ] 1.2 实现 `CommandSource::commands_for_tick()` 和 `CommandSource::is_live()`
- [ ] 1.3 实现 `SimulationDriver::is_replay()` 辅助方法
- [ ] 1.4 实现 `SimulationDriver::new_live()` 和 `SimulationDriver::new_replay(replay)` 构造方法

## 2. 统一驱动系统

- [ ] 2.1 实现 `simulation_driver_system`：累积器推进、命令获取、注入、run_tick 调用、Bevy CommandBuffer 清理
- [ ] 2.2 实现 `handle_seek`：向后 seek（重新初始化世界）、向前 seek（当前位置推进）、分帧 500 tick、accumulator 清零
- [ ] 2.3 实现录制拦截：Live + 录制开启 + 非 seek 时保存命令到 ReplayRecorder
- [ ] 2.4 实现 `inject_commands` 辅助函数：将命令注入 simulation CommandBuffer

## 3. 迁移与清理

- [ ] 3.1 在 `bevy_adapter/src/lib.rs` 中注册 `simulation_driver_system`，用 `.before(sync_entities_system)` 显式排序
- [ ] 3.2 删除 `tick_driver_system`（从 tick.rs）
- [ ] 3.3 删除 `replay_tick_driver_system` 和 `ReplayController`（从 replay.rs）
- [ ] 3.4 删除 `GameMode` 枚举（从 replay.rs）
- [ ] 3.5 更新 bevy_adapter Plugin：移除旧系统注册，添加 SimulationDriver 资源初始化
- [ ] 3.6 更新 `ReplayStatus`：`is_replay` 标注为展示态缓存

## 4. render_view 适配

- [ ] 4.1 更新 `render_view/src/lib.rs`：reset_game_system 使用 SimulationDriver 替代 GameMode
- [ ] 4.2 更新 `render_view/src/lib.rs`：cleanup_playing_system 清理 SimulationDriver
- [ ] 4.3 更新 `render_view/src/ui/replay_player.rs`：UI 控制改为操作 SimulationDriver.scheduler
- [ ] 4.4 更新 `render_view/src/ui/mod.rs`：run_if 条件使用 SimulationDriver::is_replay()
- [ ] 4.5 更新 `render_view/src/lib.rs`：replay_seeking 条件使用 SimulationDriver

## 5. 测试

- [ ] 5.1 编写 `test_speed_determinism`：通过 SimulationDriver 推进不同调度密度（1x vs 4x），验证最终状态一致
- [ ] 5.2 编写 `test_seek_determinism`：seek 后继续播放与连续播放结果一致
- [ ] 5.3 编写 `test_command_single_consumption`：Live 命令每 tick 只消费一次
- [ ] 5.4 编写 `test_seek_clears_accumulator`：seek 后 accumulator 为 0
- [ ] 5.5 运行 `cargo test -p simulation` 确认 93+ 测试通过
- [ ] 5.6 运行 `cargo test -p bevy_adapter` 确认 Driver 测试通过
- [ ] 5.7 运行 `cargo test replay` 确认 Replay 回归通过
- [ ] 5.8 运行 `cargo check` 确认全项目编译通过

---

## Post-Implementation Workflow

1. **Verify**: Run `/myspec-verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm
3. **Merge**: After user accepts, go to main branch and merge
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/unify-tick-driver`

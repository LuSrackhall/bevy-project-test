## Context

回放模式（Replay）中，`bevy_adapter::driver` 的 `simulation_driver_system` 在每 20 tick 做 hash 对比时发现 DESYNC。症状：

- tick 4040 首次出现（首次报告）
- tick 340 出现（早期退出场景）
- tick 1000 出现（ESC→主菜单退出场景）
- tick 10240 出现（长对局）

核心问题：给定相同的 `(seed, map_size, commands_per_tick)`，回放产生了与录制时不同的世界状态。深度诊断后确定了两个根因。

### 诊断过程与结论

通过三个 driver 层集成测试证明仿真层是确定性的：
- `test_driver_live_replay_determinism` — 15000 tick (seed 42/77, Small + Medium)
- `test_replay_seek_continuation_determinism` — forward seek
- `test_replay_backward_seek_determinism` — backward seek + reinit

**结论：仿真层 + 命令注入路径 + seek 路径全部确定。DESYNC 根因不在 simulation 层。**

### 根因一：Spawn Type 直接修改未被录制

`render_view::hud.rs::SpawnTypeBtn` observer **直接**修改 `CityComponent.spawn_type` 到 SimulationWorld，不推命令到 `cmd_buf`。Live 录制不包含此修改，Replay 无法回放。

**场景**：用户点击「步兵」按钮 → `c.spawn_type = SoldierType::Infantry` → 城市产出步兵 → 战斗结果不同 → 世界状态分歧。

### 根因二：回放超过 total_ticks 不停止

`simulation_driver_system` 在 `total_ticks` 处只有注释无操作。回放越过录制终点继续模拟 ghost ticks，产生虚假的 DESYNC。

**场景**：用户 ESC→主菜单退出（tick ~1050）→ replay 继续模拟 tick 1050+ → 无命令但有 AI 和仿真 → hash 分歧。

## Goals / Non-Goals

**Goals:**
- 修复 SpawnType 录制遗漏：observer 推 `SetSpawnType` 命令到 `cmd_buf`
- 修复 replay 越界：到达 `total_ticks` 自动暂停
- 新增回归测试覆盖多种规模（15000 tick, Medium 地图, 多 seed）
- 确保 replay 确定性作为联机准备

**Non-Goals:**
- 不改 `hash_world_state` 自身
- 不做联机功能本身
- 不改变 replay file 格式 v2

## Decisions

**D1: SpawnType observer 双写** — 既直接修改 simulation state（即时反馈），也推 `GameCommand{ SetSpawnType }` 到 cmd_buf（录制）。此改动不破坏现有 observer 行为。

**D2: Replay 边界处理** — `driver.scheduler.is_paused = true` 在 `current_tick >= total_ticks` 时触发。`handle_seek` 也 cap 目标为 `total_ticks`。

**D3: Driver 层测试 — 回归防护** — 3 个测试覆盖 Live→Replay、forward/backward seek，仿真层确定性已被验证。

## Risks / Trade-offs

- [SetSpawnType 重复] 双写可能导致一次 tick 内两次写同一字段 → 值相同，无害。后续可改为纯命令驱动。
- [Test 耗时] 15000 tick 测试约 9s → acceptable。

## Why

回放模式 DESYNC 存在两个根因：

1. **Spawn type 按钮**直接修改 simulation state 但不推命令，录制不包含此修改 → replay 无法回放 → 城市产出不同兵种 → 世界分歧
2. **回放过界**：`total_ticks` 后无停顿逻辑，继续模拟 ghost ticks

回放是联机的技术铺垫，必须解决。

## What Changes

- **修复** `hud.rs` spawn type observer：推 `SetSpawnType{ city, soldier_type }` 命令到 `cmd_buf`，同时保持直接修改
- **修复** `simulation_driver_system`：到达 `total_ticks` 自动 `is_paused = true`
- **修复** `handle_seek`：cap seek target 不超 `total_ticks`
- **修复** `ReplayRecorder::record_tick`：移除 `!commands.is_empty()` 过滤，无条件记录所有 tick
- **新增** 3 个 driver 层集成测试（15000 tick、Medium 地图、多 seed）
- **新增** seek 确定性测试（forward + backward）

## Capabilities

### New Capabilities
- `replay-determinism`: 回放确定性诊断与修复，提供回归测试集

### Modified Capabilities
<!-- none -->

## Impact

- **render_view**: 修改 `hud.rs` spawn type observer（推 SetSpawnType 命令）
- **bevy_adapter**: 修改 `driver.rs`（total_ticks 暂停、seek cap）、`replay.rs`（无条件录制）
- **测试**: 新增 3 个集成测试

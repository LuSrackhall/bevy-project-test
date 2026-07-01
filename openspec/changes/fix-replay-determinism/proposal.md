## Why

回放模式在运行至约 4040 tick 时出现世界状态 hash 不一致（DESYNC），持续至文件尾。DESYNC 破坏回放功能的可信度，而回放是联机功能的技术铺垫——如果录制后的回放都无法正确再现对局，联机的帧同步也无法保证。必须从根因上解决。

## What Changes

- **新增 driver 层集成测试**：模拟完整 `SimulationDriver` 流程（Live 录制 → 序列化 → 反序列化 → Replay → hash 对比），覆盖 `inject_commands` + `run_tick_default` 路径，作为回归防护
- **诊断阶段提升 hash 频率**：将 `DESYNC_CHECK_INTERVAL` 临时改为 1（每 tick hash），精确定位第一个分歧 tick
- **分歧点扩散追踪**：在每个子系统 phase 后插临时 hash，定位首次产生分歧的 phase
- **根因修复**：根据诊断结果分支修复（HashMap 迭代确定性、命令注入路径、录制过滤、AI RNG 消耗分歧等）
- **重构录制路径**（如诊断为架构问题）：移除 `!commands.is_empty()` 过滤，确保所有 tick 对齐无空洞

## Capabilities

### New Capabilities
- `replay-determinism`: 回放确定性诊断与修复，提供 driver 层集成测试作为回归防护，确保 replay 与 live 产生完全一致的世界状态

### Modified Capabilities
<!-- None — this is a repair of existing behavior, not a spec-level requirement change -->

## Impact

- **simulation**: 可能修改 `combat/mod.rs`、`soldier/mod.rs` 中的 HashMap→BTreeMap 替换（如诊断为此根因）
- **bevy_adapter**: 修改 `driver.rs`（诊断 hash 频率、命令注入路径）、`replay.rs`（录制过滤逻辑）
- **性能**: 诊断阶段每 tick hash 增加开销；正式环境恢复为 20 tick 间隔
- **测试**: 新增 driver 层集成测试，可能增加 CI 时间

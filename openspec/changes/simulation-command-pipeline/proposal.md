## Why

宪法 §2.5（统一命令驱动）已有正确原则，但实现层存在侧门：render_view observer 直接修改 SimulationWorld 绕过 CommandPipeline，导致 Replay 无法录制这些修改 → DESYNC。这是宪法 §2.5.1 的落地缺口。

更根本的问题：这不是一个 bug 修复问题，而是一个架构不变量缺失问题。后续每增加一个 observer 或系统，都需手动确保不绕过 CommandPipeline——这不可扩展。本次 Change 将 §2.5 的落地从"约定"升级为"编译期约束"，使 Replay、AI、Scenario、联机共享同一条确定性命令流水线。

## What Changes

- **P1 消除绕过路径**：SpawnType observer 删除直接修改 `c.spawn_type = btn.0`，只留 `cmd_buf.push(SetSpawnType)`
- **P2 编译期收口**：引入 `SimulationReader`（只读查询）+ `CommandSink`（命令提交），替换 render_view 对 `NonSendMut<SimulationWorld>` 的直接访问。render_view 不再持有 SimulationWorld 的可写引用
- **P3 CommandSource 统一**：消除 driver 对 CommandSource 具体类型的直接判断，用 `total_ticks() → Option<u32>` 替代 is_replay() 检查
- **宪法更新**：v1.1 新增 §1.2.7（零感知原则）、§2.5.4（Pipeline 固化）、§2.5.5（Scheduler 域盲约束）、§2.5.4（不变量）
- **Architecture Guard**：新增架构测试，CI 检查 render_view 不再引入 SimulationWorld 可写引用

## Capabilities

### New Capabilities
- `simulation-command-pipeline`: Simulation 唯一状态修改入口的架构固化。Replay、AI、Scenario、联机共享同一命令流水线。通过编译期约束和架构守卫保证未来新增功能无法绕过该流水线。

### Modified Capabilities
<!-- None — 无现存 spec 的需求级变更 -->

## Impact

- **render_view**: 移除所有 `NonSendMut<SimulationWorld>` 直接访问（只读系统改为 `SimulationReader`，下发系统改为 `CommandSink`）
- **bevy_adapter**: 定义 `SimulationReader` + `CommandSink` trait；重构 `CommandSource` 统一；消除 `is_replay()` 判断
- **simulation**: 无改动（`consume_commands_system` 已正确）
- **宪法**: v1.1 — §1.2.7, §2.5.4, §2.5.5 新增
- **ADR**: ADR-006 新增，ADR-003 Superseded

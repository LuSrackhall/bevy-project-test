## Context

simulation crate 当前约 7400 行 Rust，93 个测试分布在各模块内联 `#[cfg(test)]` 块中。现有 `golden_test.rs` 提供 4 个黄金测试验证确定性，但缺少结构化的场景测试框架。每次代码改动需人工手动验证，成本高。

宪法 `docs/constitution.md`（v1.0 Frozen）的 §10（测试与验证）、§22（CI 自动化）、§2.5（命令驱动）、§3.1（Tick 时序）已覆盖测试基础设施要求，不需要修改宪法。

## Goals / Non-Goals

**Goals:**
- 提供 Scenario + Verifier 结构化测试框架，一次编写、永久自动回归
- 修复现有宪法违规（DefaultHasher、hash 覆盖缺口、sort_tag 缺失）
- run_tick 加 RunConfig 参数支持 AI 等子系统开关

**Non-Goals:**
- 不修改宪法（docs/constitution.md 保持 Frozen）
- 不实现通用 Agent Environment / Gym 接口
- 不实现 Builder 模式（UnitId 时序问题使收益不大）

## Decisions

### RunConfig 参数

`run_tick(world, tick, config: &RunConfig)` 替代原双参数签名。`RunConfig` 是仿真初始化参数（类似 seed），不是 Tick 级命令。提供 `run_tick_default` 兼容包装。20 处调用强制迁移。

### Verifier trait 策略化验证

`Verifier::verify(&self, world: &mut World, events: &HashMap<u32, SimulationEvents>) -> Result<(), VerifyError>`。验证在 `Scenario.run()` 内部闭环完成，ScenarioOutput 不含 final_hash，Verifier 是唯一验证路径。

### hash_world_state 补齐

补齐 8 个缺失组件覆盖。补齐后现有 golden_test 预期哈希值会变，需同步更新。

### sort_tag() 显式排序标签

`Action::sort_tag()` 返回 `u8`，hardcoded 排序值。不依赖 Rust 枚举隐式判别值。

## Risks / Trade-offs

**[Risk] hash_world_state 补齐后 golden_test 哈希值变更** → 同步更新预期值，一次性破坏。

**[Risk] run_tick 签名变更影响 bevy_adapter 跨 crate 调用（9 处）** → run_tick_default 包装零行为变更。

**[Risk] Verifier 的 &mut World 可能被滥用** → trait 文档声明"不得修改 World 状态"。

**[Trade-off] 无 Builder，构造样板代码较多** → 接受。UnitId 时序问题使 Builder 收益不大。

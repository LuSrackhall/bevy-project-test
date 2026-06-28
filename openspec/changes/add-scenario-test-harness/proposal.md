## Why

每次代码改动后需要人工手动测试验证，成本高。AI 写代码后开发者充当测试人员，需求需要反复沟通，结果需要反复验证。simulation 层已是纯确定性状态机（GameCommand → run_tick → World），天然适合自动化场景测试，但缺少结构化的测试框架。同时现有测试存在宪法违规（DefaultHasher、hash 覆盖缺口、sort_tag 缺失），需要一并修复。

## What Changes

- **BREAKING** `run_tick` 签名变更：新增 `&RunConfig` 参数，提供 `run_tick_default` 兼容包装。20 处调用需迁移。
- 新增 `RunConfig` 结构体（enable_ai 字段），作为仿真初始化参数控制 AI 子系统开关。
- 新增 `Action::sort_tag()` 方法，返回显式排序标签，用于 §3.1 命令确定性排序。
- 修复 `DefaultHasher` 违规，替换为跨 Rust 版本稳定的确定性哈希函数。
- 补齐 `hash_world_state` 的 8 个缺失组件覆盖（SeekStance、SlowDebuff、FearlessBuff、ShieldComponent、AttackWindup、FacingDirection、Arrow、DroppedShield）。
- 新增 Scenario Test Harness：Scenario 结构体 + trait Verifier + 4 个内置 Verifier（SnapshotVerifier、EventVerifier、InvariantVerifier、CompositeVerifier）。
- 新增 `docs/engineering/testing.md` 工程规范文档。
- 新增 1 条 ADR（RunConfig 语义定位）。

## Capabilities

### New Capabilities
- `scenario-test-harness`: Scenario 数据结构、Verifier trait、4 个内置 Verifier、ScenarioOutput，提供可复用的场景测试框架。
- `run-config`: RunConfig 结构体及其 ADR，定义仿真运行配置参数的语义和扩展边界。

### Modified Capabilities
- `simulation-crate`: run_tick 签名变更、Action::sort_tag()、DefaultHasher 修复、hash_world_state 覆盖补齐。

## Impact

- **API 破坏**：`run_tick` 签名变更影响 simulation 内部（11 处）和 bevy_adapter（9 处）调用。
- **哈希值变更**：hash_world_state 补齐后现有 golden_test 预期哈希值会变，需同步更新。
- **无新增依赖**：不引入新 crate 依赖，Verifier trait 和 Scenario 使用现有 bevy_ecs 和 simulation 内部类型。
- **测试覆盖提升**：新增场景测试覆盖多系统联动（城市产出 + SeekStance 继承 + 移动），补充现有单元测试的集成断言缺口。

## Context

项目当前面临的核心痛点：每次代码改动后需要人工手动测试验证，成本高。AI 写代码后，开发者充当测试人员，需求需要反复沟通，结果需要反复验证。

现有测试基础设施：
- `golden_test.rs`（4 个黄金测试）+ 各模块内联测试（~95 个）
- `hash_world_state` 使用 `DefaultHasher`（违反宪法 §10.3）
- `hash_world_state` 缺少 8 个组件覆盖（SeekStance、SlowDebuff、FearlessBuff、ShieldComponent、AttackWindup、FacingDirection、Arrow、DroppedShield）
- `Action` 枚举无 `sort_tag()` 方法（宪法 §2.5 要求但代码未实现）
- 无共享测试工具模块，各模块自行定义 spawn helper
- `run_tick` 签名无 RunConfig 参数，无法控制 AI 等子系统开关

宪法 `docs/constitution.md`（v1.0 Frozen）已通过 §10、§22、§2.5、§3.1 覆盖了测试基础设施的要求，不需要修改宪法。

## Goals / Non-Goals

**Goals:**
- 将"需求是否正确实现"从人工验证转为自动断言
- 提供可复用的 Scenario 测试框架，一次编写、永久回归
- 修复现有宪法违规（DefaultHasher、hash 覆盖、sort_tag）
- 遵守宪法 §3.1 Tick 时序（命令排序）和 §17 Truth Ownership（不暴露 World）

**Non-Goals:**
- 不修改宪法（docs/constitution.md 保持 Frozen）
- 不实现通用 Agent Environment / Gym 接口（过早抽象）
- 不实现 Builder 模式（UnitId 时序问题使收益不大）
- 不实现 AI 对战测试、网络同步测试、性能基准测试（Tier 2/3 范畴）

## Decisions

### Decision 1: RunConfig 参数

**选择**：给 `run_tick` 加 `&RunConfig` 参数，提供 `run_tick_default` 兼容包装。

**理由**：RunConfig 是仿真初始化参数（类似 seed、map_size），不是 Tick 级命令，不违反 §2.5 命令驱动原则。未来可扩展（如 ai_mode），但当前仅 enable_ai 一个字段。

**放弃方案**：
- 独立函数 `run_tick_no_ai`：两套执行路径容易分叉
- 运行时拦截（移除资源）：测试黑魔法，维护成本高
- 将 enable_ai 放入 GameCommand：AI 开关不是 Tick 级决策

**代价**：`run_tick` 签名变更，21 处调用需迁移（simulation 内部 + bevy_adapter 9 处 + render_view 1 处）。

**修改条件**：若需要 Tick 内动态切换 AI，则改回 GameCommand。

### Decision 2: Scenario 纯数据结构（无 Builder）

**选择**：Scenario 为纯数据 struct，直接构造，不使用 Builder 模式。

**理由**：UnitId 在 World 创建后才能通过 query 获取，Builder 阶段无法声明式构造含 UnitId 的命令。实际用法会退化为"先建 World、查 UnitId、再构造 Scenario"，Builder 不比直接构造更方便。

**代价**：构造场景时有少量样板代码。

### Decision 3: Verifier trait 策略化验证（方向 A）

**选择**：保留 Verifier trait，ScenarioOutput 不含 final_hash。验证在 run() 内部闭环完成。

**理由**：
- 验证职责单一：run() 负责跑场景并验证，调用方只拿结果
- 不会架空 Verifier：ScenarioOutput 不含 hash，Verifier 是唯一验证路径
- 利于未来扩展：事件序列校验、不变量校验、多 verifier 组合都适合放在 Verifier 体系

**放弃方案**：
- 去掉 Verifier trait，用 ScenarioOutput 字段 + 调用方 assert：hash 字段使 Verifier 冗余
- 闭包式 verify：比 trait 简单但不如 trait 可测试

**代价**：Verifier trait 增加了一层抽象。SnapshotVerifier 对于简单 hash 比对场景略显冗余。

### Decision 4: hash_world_state 覆盖缺口处理

**选择**：本次范围内补齐 hash_world_state 的 8 个缺失组件覆盖。

**理由**：覆盖率评审指出，缺 8 个组件的 hash 会导致 SnapshotVerifier 漏报战斗/Shield/Arrow 相关回归。不补的话 harness 核心价值打折。

**代价**：需要逐个确认缺失组件的字段和哈希方式。

### Decision 5: DefaultHasher 修复

**选择**：替换 `std::collections::hash_map::DefaultHasher` 为跨 Rust 版本稳定的确定性哈希函数。

**理由**：宪法 §10.3 明确禁止使用 DefaultHasher，其哈希值随 Rust 版本变化。

## Risks / Trade-offs

**[Risk] hash_world_state 补齐后现有 golden_test 哈希值会变**
→ golden_test 使用"同种子两次运行 hash 相等"模式，不依赖硬编码预期值，实际无需更新。

**[Risk] run_tick 签名变更影响 bevy_adapter 跨 crate 调用**
→ bevy_adapter/driver.rs 有 9 处调用需迁移。通过 run_tick_default 包装可零行为变更。

**[Risk] Verifier trait 的 &mut World 参数可能被滥用修改状态**
→ 在 trait 文档中声明"Verifier 不得修改 World 状态"的契约。技术上无法强制，但 code review 可检查。

**[Trade-off] Scenario 不含 Builder，构造样板代码较多**
→ 接受。UnitId 时序问题使 Builder 收益不大。未来如需简化，可添加辅助函数而非 Builder。

**[Trade-off] hash_world_state 覆盖缺口的已知限制**
→ 在 testing.md 中声明 SnapshotVerifier 仅覆盖 hash_world_state 当前包含的组件。本次补齐后此限制解除。

## Implementation Order

1. **Action::sort_tag()** — 宪法要求，Scenario 排序依赖此方法
2. **DefaultHasher 修复** — 宪法违规，顺手清理
3. **hash_world_state 覆盖补齐** — SnapshotVerifier 的前提
4. **RunConfig + run_tick 签名变更** — 20 处调用迁移
5. **Verifier trait + 4 个内置 Verifier** — 核心抽象
6. **Scenario + ScenarioOutput + run()** — 测试框架核心
7. **testing.md** — 工程规范文档
8. **ADR（RunConfig 语义定位）** — 架构决策记录
9. **示例场景测试** — 8 个测试覆盖 SnapshotVerifier/EventVerifier/InvariantVerifier/CompositeVerifier + 命令注入 + AI 禁用

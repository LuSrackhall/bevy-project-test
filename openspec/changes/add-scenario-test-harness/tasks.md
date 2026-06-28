## 1. Action::sort_tag() 实现

- [x] 1.1 在 `crates/simulation/src/command.rs` 中为 `Action` 枚举添加 `pub const fn sort_tag(&self) -> u8` 方法，为每个变体分配显式 u8 排序标签
- [x] 1.2 添加单元测试验证 sort_tag 返回值固定且不依赖枚举隐式判别值

## 2. DefaultHasher 修复

- [x] 2.1 将 `crates/simulation/src/golden_test.rs` 中的 `DefaultHasher` 替换为跨 Rust 版本稳定的确定性哈希函数（如 `twox-hash` 或手动实现的 FNV/SipHash）
- [x] 2.2 更新 golden_test.rs 中所有 4 个测试的预期哈希值
- [x] 2.3 运行 `cargo test -p simulation` 确认所有测试通过

## 3. hash_world_state 覆盖补齐

- [x] 3.1 在 `hash_world_state` 中添加以下组件的哈希覆盖：SeekStance、SlowDebuff、FearlessBuff、ShieldComponent、AttackWindup、FacingDirection、Arrow、DroppedShield
- [x] 3.2 更新 golden_test.rs 中所有预期哈希值（覆盖变更后哈希值会变）
- [x] 3.3 运行 `cargo test -p simulation` 确认所有测试通过

## 4. RunConfig + run_tick 签名变更

- [x] 4.1 创建 `crates/simulation/src/run_config.rs`，定义 `RunConfig { enable_ai: bool }` 和 `Default` 实现
- [x] 4.2 修改 `run_tick` 签名为 `(world: &mut World, tick_number: u32, config: &RunConfig) -> SimulationEvents`
- [x] 4.3 添加 `run_tick_default(world, tick)` 兼容包装
- [x] 4.4 在 `run_tick` 内部 ai_decide 阶段添加 `if config.enable_ai` 条件判断
- [x] 4.5 迁移 simulation crate 内部 11 处 `run_tick` 调用为 `run_tick_default`
- [x] 4.6 迁移 `crates/bevy_adapter/src/driver.rs` 中 9 处 `simulation::run_tick` 调用为 `simulation::run_tick_default`
- [x] 4.7 运行 `cargo test` 全项目确认无编译错误和测试失败

## 5. Verifier trait 与内置 Verifier

- [x] 5.1 创建 `crates/simulation/src/scenario/` 目录结构（mod.rs, verifier.rs, verifiers/）
- [x] 5.2 实现 `VerifyError` 枚举（HashMismatch, EventMismatch, InvariantViolation, Composite），包含 source 和 tick 字段
- [x] 5.3 实现 `trait Verifier { fn name(&self) -> &'static str; fn verify(...) -> Result<(), VerifyError> }`
- [x] 5.4 实现 `SnapshotVerifier`（调用 hash_world_state 比对）
- [x] 5.5 实现 `EventVerifier`（builder API: expect_spawned_at, expect_captured_at 等）
- [x] 5.6 实现 `InvariantVerifier`（接受闭包列表）
- [x] 5.7 实现 `CompositeVerifier`（组合多个 verifier，收集所有错误）
- [x] 5.8 为每个 Verifier 编写单元测试

## 6. Scenario + ScenarioOutput + run()

- [x] 6.1 实现 `Scenario` 结构体（seed, map_size, config, commands, max_tick, verifier）
- [x] 6.2 实现 `ScenarioOutput`（events_per_tick: HashMap<u32, SimulationEvents>）
- [x] 6.3 实现 `Scenario::run()` 方法：init_simulation_world + generate_map + 按 tick 分组 + 排序 + 注入 + run_tick + verify
- [x] 6.4 在 lib.rs 中导出 scenario 模块
- [x] 6.5 编写首个示例场景测试（城市产出 + SeekStance 继承 + 移动，300 tick）
- [x] 6.6 运行 `cargo test -p simulation` 确认所有测试通过

## 7. 文档与 ADR

- [ ] 7.1 创建 `docs/engineering/testing.md`，定义 scenario harness 使用约定、Verifier 编写规范、CI 集成方式
- [ ] 7.2 创建 `docs/adr/0001-run-config-semantics.md`，记录 RunConfig 的语义定位（决策、放弃方案、代价、修改条件）
- [ ] 7.3 提交所有文档

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/add-scenario-test-harness`

**Iteration**: If user does not accept, analyze the issue and recommend:
fix in place / new change / git reset + stash / git reset / abandon.

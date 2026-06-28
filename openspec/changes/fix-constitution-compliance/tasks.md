## 1. 组 A：小修复

- [x] 1.1 删除 `types.rs` 中 `gen_probability()` deprecated 方法，确认无调用方
- [x] 1.2 替换 `bevy_adapter/src/driver.rs` 中 `world_fingerprint` 的 DefaultHasher 为 FNV-1a
- [x] 1.3 补齐 `golden_test.rs` 中 hash_world_state 的字段覆盖：Movement（command_target, waypoint）、CityComponent（max_level, spawn_type, last_attacker_faction, arrow_damage_acc）、CityOrigin 组件、SoldierStateComponent 组件
- [x] 1.4 运行 `cargo test -p simulation` 确认所有测试通过

## 2. §3.1 Tick 时序：ReplayFile + consume_commands_system

- [ ] 2.1 给 `replay.rs` 中 ReplayFile 加 `derive(bevy_ecs::prelude::Resource)`
- [ ] 2.2 修改 `soldier/mod.rs` 中 `consume_commands_system` 签名：从 `fn(world, tick)` 改为 `fn(world, commands: Vec<GameCommand>)`，删除内部 take_for_tick 调用
- [ ] 2.3 适配 soldier/mod.rs 中 4 个 seek_stance 测试（改为直接构造 Vec 传入）
- [ ] 2.4 运行 `cargo test -p simulation` 确认编译通过

## 3. §3.1 Tick 时序：run_tick 六步流程

- [ ] 3.1 在 `lib.rs` 中新增 `collect_command_players(world) -> Vec<u8>` 函数（显式 match，仅 Player/Enemy）
- [ ] 3.2 在 `run_tick` 中实现六步流程：Step 1 take_for_tick → Step 2 NoOp 注入 → Step 3 排序 → Step 4 归档（可选 ReplayFile） → Step 5 清除事件 + consume_commands_system + 其余 Phase + ai_decide → Step 6 返回 SimulationEvents
- [ ] 3.3 修改 `scenario/mod.rs` 中 `Scenario::run()`：删除自行排序（run_tick 已处理），命令直接 extend 进 CommandBuffer
- [ ] 3.4 运行 `cargo test -p simulation` 确认所有测试通过

## 4. CI 自动化检查

- [ ] 4.1 在 `.github/workflows/ci.yml` 中添加 simulation 禁用类型扫描步骤（grep bevy_render/bevy_window/bevy_ui/bevy_input/bevy_asset/bevy_math）
- [ ] 4.2 添加浮点渗入检测步骤（grep simulation crate 中非白名单的 f32/f64 使用）
- [ ] 4.3 添加 hash_world_state 覆盖率检查步骤（比对组件列表）
- [ ] 4.4 添加依赖拓扑检查步骤（simulation Cargo.toml 不依赖下游 crate）
- [ ] 4.5 运行全量测试确认 CI 配置正确

## 5. 最终验证

- [ ] 5.1 运行 `cargo test` 全项目确认无编译错误和测试失败
- [ ] 5.2 检查 §3.1 六步流程在 run_tick 中完整实现
- [ ] 5.3 检查 Scenario::run() 不再自行排序

---

## Post-Implementation Workflow

After completing ALL tasks above, follow this sequence strictly:

1. **Verify**: Run `/opsx:verify` to produce verify.md
2. **User Acceptance**: Present change summary, ask user to confirm the problem is solved
3. **Merge**: After user accepts, go to main branch and merge (must ask user)
4. **Archive**: Run `/opsx:archive` on main
5. **Cleanup**: `git worktree remove .worktrees/change/fix-constitution-compliance`

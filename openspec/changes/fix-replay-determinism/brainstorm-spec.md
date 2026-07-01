## Context

回放模式（Replay）中，`bevy_adapter::driver` 的 `simulation_driver_system` 在每 20 tick 做 hash 对比时发现 DESYNC。已知的未解决案例从 tick 4040 开始持续不一致：

```
DESYNC at tick 4040: replay hash 16154204828727490913 != recorded hash 9984251738854007026
```

核心问题：给定相同的 `(seed, map_size, commands_per_tick)`，回放产生了与录制时不同的世界状态。这破坏回放功能的可信度，且回放作为联机的技术铺垫，这个问题必须从根本上解决。

已排除的根因：

- `run_tick_default` 本身是确定性的——`golden_test` 和 `replay::test_e2e_replay_determinism` 在纯仿真测试中通过
- `SpatialHash` 使用 `BTreeMap`，遍历顺序是确定性的
- `DeterministicRng`（`SmallRng::seed_from_u64`）是确定性的

疑点区域：

~~1. **命令注入路径差异**：... [已排除 — 5000 tick 集成测试通过]~~
~~2. **录制过滤**：... [已修复 — ReplayRecorder 无条件记录所有 tick]~~
~~3. **仿真层 HashMap 迭代**：... [已排除 — 5000 tick 集成测试通过]~~
~~4. **AI 决策 RNG 消耗量分歧**：... [已排除 — 5000 tick 集成测试通过]~~

**已排除的疑点区域（通过 5000 tick driver 集成测试 + seek 测试）**：仿真层、命令注入路径和 seek 路径全部是确定性的。

**未排除**：bevy 帧调度层的系统交互、特定 replay 文件在 tick 4040 附近的 Entity 组合、future-frame `NonSendMut<SimulationWorld>` 只读访问的时序影响。需要用户回放特定 replay 文件并配合诊断日志进一步定位。

## Goals / Non-Goals

**Goals:**
- 精确诊断回放 DESYNC 的根因，定位到具体的子系统/代码行
- 修复根因后，回放运行到文件尾（约 5000+ tick）零 DESYNC
- 新增回归测试，覆盖完整 driver 流程（不依赖 bevy frame），作为长期防护
- 借此机会修正 replay/recording 架构中可能影响未来联机正确性的问题

**Non-Goals:**
- 不改 `hash_world_state` 自身（它是正确的参考点）
- 不做联机功能本身——只修复回放确定性，为联机做准备
- 不改变 replay file 格式 v2（但可添加字段，必须向后兼容）

## Decisions

### D1: 诊断流程 — 三步定位法

**Step 1: 精确分歧点定位**

将 hash 记录频率从 `DESYNC_CHECK_INTERVAL=20` 改为每 tick 记录（仅诊断阶段），定位第一个不一致的 tick。

**Step 2: Driver 层集成测试**

新建 `test_driver_live_replay_determinism`，模拟完整 `SimulationDriver` 流程：

```
Live 录制（N=5000 tick，AI + 随机人工命令）
→ 序列化 ReplayFile
→ 反序列化
→ Replay 回放
→ 逐 tick 对比 hash
```

关键差异——不同于现有 `test_e2e_replay_determinism`（直接 `run_tick_default`），本测试通过 `inject_commands` + `run_tick_default` 双调用，暴露 driver 层问题：

- 若通过 → 问题在 bevy 帧时序层面（accumulator 偏移、`NonSendMut` 跨帧状态残留等）
- 若失败 → 问题在仿真或命令注入路径，本测试即为回归防护

**Step 3: 分歧点扩散追踪**

从第一个分歧 tick 开始，在 `run_tick` 中每个子系统 phase 后插临时 hash 调用，定位第一个产生分歧的相位：

```
Phase 1: consume_commands → hash
Phase 2: combat_engagement → hash
Phase 3: facing_turn → hash
Phase 4: soldier_movement → hash
...
Phase N: AI → hash
```

### D2: 修复策略（根据诊断结果分支）

| 诊断结论 | 修复方案 |
|---|---|
| **仿真层 HashMap 迭代非确定** | 将 `combat/mod.rs` 和 `soldier/mod.rs` 中只用作构建后不修改的 HashMap 替换为 BTreeMap，或在迭代处排序 |
| **Driver 命令注入时序差异** | 重构 `inject_commands` → `run_tick` → `take_for_tick` 路径，确保 Live 和 Replay 的 `CommandBuffer` 流顺序完全一致 |
| **录制过滤导致 tick 空洞** | `ReplayRecorder::record_tick` 移除 `!commands.is_empty()` 过滤，改为记录每个 tick 的命令列表（含空），保证 tick 对齐无空洞 |
| **Bevy 帧时序 / accumulator 偏移** | 在 `handle_seek` 和 `simulation_driver_system` 中增加 tick 边界校验，确保回放与录制 tick 计数一致 |
| **World::query 迭代顺序变化** | 所有影响仿真状态的系统遍历改为按 UnitId 排序 |

### D3: 重构原则（为联机做准备）

如果发现架构问题需要重构，遵循以下原则：

1. **命令流 100% 确定**：`record_tick` 记录的命令必须是 `inject_commands` 收到的精确副本，不能有过滤或跳过
2. **无条件录制**：移除 `!commands.is_empty()` 过滤——空命令 Vec 也要记录，保证 tick 对齐
3. **Driver 层可测试化**：`simulation_driver_system` 中 Live→record→Replay→verify 的闭环应能通过纯测试（不依赖 bevy frame）验证
4. **RNG 消耗审计**：`ai_decide` 中每次 RNG 调用必须由相同输入产生，Entity 遍历顺序变化不能改变 RNG 消耗量
5. **非确定性结构审查**：逐个审查 simulation 中所有 `HashMap` 的使用，确保其 `iter()` 顺序不影响仿真输出；对于仅用于构建后只读查询的 map，优先使用 `BTreeMap`

## Risks / Trade-offs

- **[诊断精度] 每 tick hash 增加性能开销** → 仅诊断阶段启用，正式环境恢复为 20 tick 间隔
- **[回归测试] 新增 driver 集成测试耗时较长**（5000 tick × 20Hz = 250 秒仿真时间）→ 测试编译为 release 模式或减小 tick 数；存在即可
- **[过度重构] 可能一次性改动过大** → D2 的分支策略确保只做最小必要修复；D3 的原则作为守卫，不盲目重构
- **[HashMap→BTreeMap 性能影响] 全量替换可能导致 O(log N) 降速** → 只在迭代影响仿真状态的路径上替换；仅用于 O(1) 查找的保持 HashMap

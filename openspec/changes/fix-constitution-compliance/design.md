## Context

宪法合规审计发现 5 项违规需要修复。其中 §3.1 Tick 时序是最核心的——run_tick 当前缺少三个步骤（No-Op 注入、命令排序、命令归档），需要在 run_tick 内部完整实现六步流程。

## Goals / Non-Goals

**Goals:**
- 完全实现 §3.1 六步 Tick 时序
- 修复全部 PARTIAL 违规
- 保持确定性

**Non-Goals:**
- 不修复 §4.2 combat O(n²)（Tier 2）
- 不修复 §5.5 + §17 render_view（ADR-003）
- 不修改宪法

## Decisions

### run_tick 六步流程

Step 1-4 在仿真前完成（收集→补齐→排序→归档），Step 5 执行仿真（清除事件→consume_commands_system→其余 Phase→ai_decide），Step 6 返回 SimulationEvents。

consume_commands_system 签名从 `fn(world, tick)` 改为 `fn(world, commands: Vec<GameCommand>)`，不再自行 take_for_tick。

collect_command_players 使用显式 match（Faction::Player→0, Enemy→1, Neutral→排除），不使用 `as u8`。

ReplayFile 加 derive(Resource) 作为可选归档目标。

### 组 A 小修复

gen_probability 删除、DefaultHasher 替换、hash 字段补齐、CI 检查补齐。

## Risks / Trade-offs

**[Risk] consume_commands_system 签名变更** → 4 个 seek_stance 测试需适配

**[Risk] 排序改变多玩家命令顺序** → 确定性改进的必然代价，旧 replay 需重录

**[Risk] NoOp 注入改变行为** → Action::NoOp 是空操作，零影响

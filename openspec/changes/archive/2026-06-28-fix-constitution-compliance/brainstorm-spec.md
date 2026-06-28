## Context

宪法合规审计发现以下违规：

| 条款 | 判定 | 问题 |
|------|------|------|
| §3.1 Tick 时序 | **FAIL** | run_tick 缺少 No-Op 注入、命令排序、命令归档三个步骤 |
| §2.3 数值规范 | **PARTIAL** | `gen_probability()` 返回 f32 的 deprecated 方法未删除 |
| §10.2 hash 覆盖 | **PARTIAL** | Movement 缺 command_target/waypoint，CityComponent 缺 4 个字段，CityOrigin/SoldierStateComponent 未覆盖 |
| §10.3 确定性哈希 | **PARTIAL** | bevy_adapter/driver.rs 残留 DefaultHasher |
| §22 CI 自动化 | **PARTIAL** | 缺少禁用类型扫描、浮点渗入检测、依赖拓扑检查、hash 覆盖率检查 |
| §4.2 禁止热点 O(n²) | **PARTIAL** | combat 系统全量扫描（Tier 2 范畴，本次不修） |
| §5.5 + §17 | **FAIL** | render_view 直接操作 SimulationWorld（ADR-003 已记录为阶段一许可，本次不修） |

## Goals / Non-Goals

**Goals:**
- 完全实现 §3.1 六步 Tick 时序（收集→补齐→排序→归档→仿真→输出）
- 修复全部 PARTIAL 违规（gen_probability、hash 覆盖、DefaultHasher、CI）
- 保持确定性：同 seed + 同命令 + 同 RunConfig = 同结果

**Non-Goals:**
- 不修复 §4.2 combat O(n²)（Tier 2 触发后再处理）
- 不修复 §5.5 + §17 render_view 违规（ADR-003 阶段一许可）
- 不修改宪法

## Decisions

### Decision 1: run_tick 内部实现完整六步流程

**选择**：将 §3.1 六步流程全部实现在 run_tick 内部，不绕过任何步骤。

**具体改动：**

1. **Step 1 指令收集**：run_tick 内部调用 `CommandBuffer::take_for_tick`
2. **Step 2 指令补齐**：新增 `collect_command_players(world)` 函数，从 FactionComponent 推导已知玩家（仅 Player=0, Enemy=1，显式 match 映射，排除 Neutral），缺失者注入 Action::NoOp
3. **Step 3 指令排序**：`commands.sort_by_key(|c| (c.player_id, c.action.sort_tag()))`
4. **Step 4 指令归档**：ReplayFile 加 `derive(Resource)`，作为可选 Resource 插入 World。存在时调用 `record_tick`，不存在时跳过
5. **Step 5 确定性仿真**：清除 SimulationEvents → `consume_commands_system(world, commands)` 接收已排序命令（签名变更）→ 其余 13 个 Phase 不变 → ai_decide（受 RunConfig 控制）
6. **Step 6 状态输出**：clone SimulationEvents

**放弃方案**：
- 排序放在 consume_commands_system 内部：宪法要求排序在执行前完成，应由 run_tick 统一管控
- 归档放在 run_tick 外部：宪法将归档列为 Tick 时序的一部分，应在 run_tick 内部

**代价**：
- consume_commands_system 签名变更：从内部 take_for_tick 改为接收外部 `Vec<GameCommand>`
- 4 个 seek_stance 测试需适配新签名
- ReplayFile 需加 `derive(Resource)`
- Scenario::run() 删除自行排序（run_tick 已处理）

### Decision 2: ReplayFile 作为可选 Resource

**选择**：给 ReplayFile 加 `derive(bevy_ecs::prelude::Resource)`，run_tick 通过 `world.get_mut::<ReplayFile>()` 可选访问。

**理由**：归档是 §3.1 的必须步骤，但不是所有场景都需要录制。可选 Resource 模式既满足"run_tick 内部归档"的要求，又不强制每个场景都插入 ReplayFile。

**代价**：ReplayFile 的 derive 列表变更，replay 测试需要将 ReplayFile 插入 World。

### Decision 3: collect_command_players 使用显式 match

**选择**：Faction→player_id 映射使用 `match f.0 { Faction::Player => 0, Faction::Enemy => 1, Faction::Neutral => {} }`，不使用 `as u8`。

**理由**：宪法 §2.5 禁止依赖 Rust 枚举隐式判别值。显式 match 更安全、更清晰。

### Decision 4: hash_world_state 字段补齐

**选择**：补齐 Movement（command_target, waypoint）和 CityComponent（max_level, spawn_type, last_attacker_faction, arrow_damage_acc）的字段覆盖，以及 CityOrigin 和 SoldierStateComponent 组件覆盖。

**代价**：golden_test 哈希值会变化（但 golden_test 使用比较模式，不依赖硬编码值）。

### Decision 5: DefaultHasher 替换

**选择**：bevy_adapter/driver.rs 的 `world_fingerprint` 函数中的 DefaultHasher 替换为 FNV-1a（与 golden_test 一致）。

### Decision 6: gen_probability() 移除

**选择**：删除 `types.rs` 中的 `gen_probability()` deprecated 方法。确认无调用方后移除。

### Decision 7: CI 自动化检查补齐

**选择**：在 `.github/workflows/ci.yml` 中添加 4 个检查步骤：
1. simulation 禁用类型扫描（grep bevy_render/bevy_window 等）
2. 浮点渗入检测（grep f32/f64 在非白名单上下文）
3. hash_world_state 覆盖率检查（比对组件列表）
4. 依赖拓扑检查（simulation Cargo.toml 不依赖下游 crate）

## Risks / Trade-offs

**[Risk] consume_commands_system 签名变更影响 4 个测试** → 适配测试，构造 Vec 传入

**[Risk] 排序步骤可能改变多玩家同 tick 命令的执行顺序** → 旧 replay 文件需重新录制，这是确定性改进的必然代价

**[Risk] NoOp 注入可能改变现有行为** → Action::NoOp 是空操作（`{}`），对仿真状态零影响

**[Risk] ReplayFile derive(Resource) 可能影响序列化** → Resource 是 marker trait，不影响 Serialize/Deserialize

**[Trade-off] 归档作为可选步骤而非强制** → 宪法要求归档，但不强制要求每个 run_tick 都录制。可选 Resource 是务实的选择

## Implementation Order

### 组 A：4 项小修复
1. 删除 gen_probability() deprecated 方法
2. bevy_adapter DefaultHasher → FNV-1a
3. hash_world_state 字段补齐
4. CI 自动化检查补齐

### 组 B：§3.1 Tick 时序完整实现
5. ReplayFile 加 derive(Resource)
6. consume_commands_system 签名变更（接收外部 Vec）
7. run_tick 实现六步流程
8. Scenario::run() 删除自行排序
9. 4 个 seek_stance 测试适配
10. 全量测试验证

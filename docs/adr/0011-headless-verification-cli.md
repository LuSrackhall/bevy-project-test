# ADR 0011: 无头验证 CLI 与机器验收门

## 状态

**Date**: 2026-09-12
**Status**: Accepted（实现于 P0「agent 原生验证闭环」）

## 背景

项目长期存在一个结构性瓶颈：**验证权在人手里**。

- 仿真侧的验证能力其实早已具备（`scenario` harness、`hash_world_state`、回放、212 个测试），
  但它们只能从 Rust 测试内部调用——agent 想验证一个改动，必须先写 Rust 测试。
- 工作流（`myspec-*` / `openspec`）明文要求「用户验收」检查点，人类成为必经闸门。
- CI 名义上是门禁，实际长期失败：`Format check`（fmt 漂移）与 `Clippy`
  在 main 上连续红灯（GitHub Actions 历史可查），宪法 §22 承诺的自动化守护从未真正生效。

结果是：改动的边际成本由「人跑一次验证」决定，项目在 2026-08 后停滞。

## 决策

1. **新增独立二进制 `sim-cli`（`crates/sim-cli`）**，作为面向 agent 的无头验证入口：
   - `scenario`：跑 seed+map+ticks 的确定性仿真；默认 `repeat=2` 内置确定性门；
     支持 `--expect-hash` 黄金哈希断言、`--require-decided` 对局决出断言、
     `--record` 落盘回放并**立即重放比对逐检查点哈希**。
   - `replay`：重放回放文件，按 `DESYNC_CHECK_INTERVAL`（20 tick）比对已记录哈希；
     格式版本不兼容时快速失败（§20.2）。
   - `selfplay`：AI 对被动方跑到 tick 上限，报告胜负、阵营统计与终态哈希。
   - 统一 `--json` 输出 + 退出码（`0` 通过 / `1` 验证失败 / `2` 用法或格式错误）。
   - **不依赖 Bevy 渲染栈**，只依赖 `simulation` + `serde_json`，使 agent 迭代循环的
     编译成本与渲染解耦（冷编译实测 33s / 115M，对比全壳 14G target）。
2. **CI 机械门禁（§22）落地**：
   - 修复 `fmt` 与 `clippy -D warnings` 存量问题，使门禁可绿；
   - 修复浮点守卫**误报**：原先靠「同一行是否含 `from_float`」放行，但函数体那两行不含该词，
     必然误报。改为**显式白名单标记** `CONSTITUTION-ALLOW`（签名行仍按函数名豁免）；
   - 新增 `hash_world_state` 覆盖率守卫（`scripts/check-hash-coverage.py`），
     取代原先 CI 中的 `TODO`；
   - 新增 `Machine acceptance — headless scenario` 步骤（Linux），
     以 `sim-cli scenario --repeat 2 --record …` 作为机器验收门。
3. **补齐 `hash_world_state` 覆盖率**：纳入 `CityRadius`、`AuraHealComponent`、
   `SpawnDirection` 三个此前未参与哈希的仿真状态（§10.2 要求覆盖所有影响结果的组件）。

## 放弃的方案

- **只写 Rust 测试、不做 CLI**：agent 每次验证都要写/改 Rust 代码，且无法把
  「同输入哈希一致」「回放逐检查点一致」表达为可组合的退出码，闭环成本不降。
- **把 CLI 依赖 `bevy_adapter`**：会让 agent 循环重新绑定渲染栈的编译成本，
  违背「仿真核与外壳解耦」的既有架构意图。
- **在 CI 里硬编码黄金哈希常量**：项目现状没有任何硬编码黄金值，且仿真仍在快速演进；
  硬编码会让每次合法行为变更都要改常量，制造噪声。改为「确定性 + 回放往返」门。
- **改哈希时顺带 bump `ReplayFile` 版本号**：实测表明仓库内 8 月的回放（v2）
  在当前代码下**本就已 desync**（`sim-cli replay` 比对 684/439 个检查点，
  首个失配分别在 tick 60 / 6740），说明其哈希早已失效；再 bump 版本只会
  让这些文件从「可加载但失配」变成「直接拒绝加载」，收益不明确。

## 代价

- `hash_world_state` 覆盖补齐后**黄金哈希值改变**（`6342140384826867723` →
  `16188193490060276267`）：所有以旧哈希为准的记录（含仓库内 v2 回放的
  `tick_hashes`）永久失效。上文的实测已证明它们此前就已失效。
- 随后 AI 进攻逻辑修复（见下文「后续修订」）再次改变行为，当前黄金哈希为
  `3052343124539132435`。
- `sim-cli` 引入一个新依赖（`serde_json`）与一个新的独立二进制；按 §21
  属「独立二进制」类，不进入 `simulation` 依赖图，也不新增 feature flag。
- `too_many_arguments` 的三处（`RelayServer::new`、`udp_session`、
  `lobby_update_system`、`update_room_list`）以显式 `#[allow]` + 说明豁免，
  未做参数结构体重构。

## 后续修订（同日）

本 ADR 初稿把 `selfplay` 记为「非对称，AI 无法自对弈」。随后在实现 P0 的
「AI 自对弈」验收门时发现根因并修复，故 `limitation` 字段已删除：

- **AI 死锁（行为缺陷）**：扩张/进攻的门槛是「目标城附近已有己方士兵」
  （`ai_nearby > 0`），而士兵初始都待在本城 → 第一波永远派不出去。
  实测（seed=42, Small）：修复前 4000 与 12000 tick 下 `spawned` 恒为 60、
  `destroyed/captured/damage` 恒为 0，世界在初始产兵后彻底冻结；
  修复后 4000 tick 内 `destroyed=173 captured=8`，AI 全取 6 城。
- **敌方选择泛化**：从硬编码 `FactionId(0)` 改为「所有非己方、非中立阵营」，
  使 AI 之间可互相对抗（对称自对弈）。
- **忙碌判定修正**：AI 原先用 `mov.target.is_some()` 判断士兵已派遣，但
  `MoveTo` 设置的是 `waypoint`（见 `soldier::apply_movement`）→ 正在行军的
  士兵被误判为空闲，每 40 tick 重复下发同一命令。改为
  `target.is_some() || waypoint.is_some()`。
- **阵营标识修正**：低血城池换兵种分支硬编码 `player_id: 1`，导致非 faction 1
  的 AI（例如对称自对弈中的 faction 0）命令会被 `validate_commands` 拒绝。

由此 `selfplay --symmetric` 成为真正的对称自对弈，并新增三条机器门禁：
`ai_prosecutes_offensive_within_budget`、`symmetric_selfplay_reaches_decision`、
`selfplay_is_deterministic`，以及 CI 步骤
`Machine acceptance — AI self-play must reach a decision`。

## 修改条件

满足以下任一条件时应重新评估本决策：

1. ~~AI 改造为「以最近敌方阵营为目标」后…~~ **已满足**（见「后续修订」）：
   对称自对弈的**存在性**已可作门禁。下一步若要把「AI 策略质量」也纳入门禁
   （例如要求胜者在合理 tick 内决出、或对局不得退化为互不接触），需先定义
   可机器判定的策略指标。
2. 引入 `bevy_remote`（BRP）运行时可观测后（P0.4），
   `sim-cli` 的职责边界需重新划分：静态/回放验证留在 CLI，
   运行时状态断言移交 BRP，避免两套入口语义重叠。
3. 若 `sim-cli` 的 JSON schema 成为外部消费者（CI 之外的 agent 工具链）的契约，
   需为其引入版本号并纳入 §20 序列化契约。

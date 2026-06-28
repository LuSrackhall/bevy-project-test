# scenario-test-harness Specification

## Purpose
TBD - created by archiving change add-scenario-test-harness. Update Purpose after archive.
## Requirements
### Requirement: Scenario 数据结构

Scenario SHALL 定义纯数据结构，包含 seed、map_size、config（RunConfig）、commands（Vec<GameCommand>）、max_tick、verifier（Box<dyn Verifier>）。Scenario SHALL NOT 使用 Builder 模式。

#### Scenario: 构造基本场景

- **WHEN** 构造一个 Scenario { seed: 42, map_size: MapSize::Small, config: RunConfig::default(), commands: vec![], max_tick: 100, verifier: Box::new(SnapshotVerifier::hash(EXPECTED)) }
- **THEN** 所有字段可正确设置，Scenario 为纯数据结构

### Requirement: Scenario.run() 执行流程

Scenario.run() SHALL 返回 `Result<ScenarioOutput, VerifyError>`。内部流程 SHALL 按以下顺序执行：(1) init_simulation_world(seed) 创建 World；(2) generate_map(&mut world, map_size) 生成地图；(3) 按 tick 分组 commands；(4) 循环 tick 1..=max_tick：收集命令、按 (player_id, action.sort_tag()) 排序、注入 CommandBuffer、调用 run_tick、收集 SimulationEvents；(5) 调用 verifier.verify() 验证；(6) 返回 ScenarioOutput。

#### Scenario: 空命令场景执行

- **WHEN** 执行 Scenario { seed: 42, map_size: MapSize::Small, commands: vec![], max_tick: 100, verifier: SnapshotVerifier }
- **THEN** run() 不 panic，返回 Ok(ScenarioOutput)

#### Scenario: 验证失败返回 Err

- **WHEN** 执行 Scenario 且 verifier.verify() 返回 Err
- **THEN** run() 返回 Err(VerifyError)，错误信息包含 verifier 名称

#### Scenario: 命令按 sort_tag 排序

- **WHEN** 同一 tick 注入多条不同 player_id 的命令
- **THEN** 命令在注入 CommandBuffer 前按 (player_id, action.sort_tag()) 字典序升序排序

### Requirement: ScenarioOutput 不含 World

ScenarioOutput SHALL 包含 events_per_tick: HashMap<u32, SimulationEvents>。ScenarioOutput SHALL NOT 包含 World 所有权或引用。

#### Scenario: World 不泄露

- **WHEN** Scenario.run() 返回 Ok(ScenarioOutput)
- **THEN** 调用方无法访问原始 World 对象

### Requirement: Verifier trait

Verifier trait SHALL 定义 `fn name(&self) -> &'static str` 和 `fn verify(&self, world: &mut World, events: &HashMap<u32, SimulationEvents>) -> Result<(), VerifyError>`。Verifier 不得修改 World 状态。

#### Scenario: Verifier 通过 name() 返回标识

- **WHEN** 调用 SnapshotVerifier::hash(0).name()
- **THEN** 返回非空 &'static str

### Requirement: SnapshotVerifier

SnapshotVerifier SHALL 调用 hash_world_state 并与预期值比对。失败时返回 VerifyError::HashMismatch，包含 expected、actual 和 source 字段。

#### Scenario: Hash 匹配通过

- **WHEN** SnapshotVerifier::hash(EXPECTED) 在 hash(world) == EXPECTED 时执行 verify()
- **THEN** 返回 Ok(())

#### Scenario: Hash 不匹配失败

- **WHEN** SnapshotVerifier::hash(EXPECTED) 在 hash(world) != EXPECTED 时执行 verify()
- **THEN** 返回 Err(VerifyError::HashMismatch { expected, actual, source })

### Requirement: EventVerifier

EventVerifier SHALL 提供 builder API（expect_spawned_at、expect_captured_at 等），按 tick 校验 SimulationEvents 中的事件。失败时返回 VerifyError::EventMismatch。

#### Scenario: 事件匹配通过

- **WHEN** EventVerifier::new().expect_spawned_at(10, |s| s.len() > 0) 在 tick 10 有 spawned 事件时执行 verify()
- **THEN** 返回 Ok(())

#### Scenario: 事件不匹配失败

- **WHEN** EventVerifier::new().expect_spawned_at(10, |s| s.len() > 0) 在 tick 10 无 spawned 事件时执行 verify()
- **THEN** 返回 Err(VerifyError::EventMismatch { tick: 10, detail, source })

### Requirement: InvariantVerifier

InvariantVerifier SHALL 接受 Vec<Box<dyn Fn(&mut World) -> Option<String>>> 闭包列表，逐个执行并在首个返回 Some 时失败。

#### Scenario: 不变量全部满足

- **WHEN** InvariantVerifier 的所有闭包返回 None
- **THEN** 返回 Ok(())

#### Scenario: 不变量违反

- **WHEN** InvariantVerifier 的某个闭包返回 Some(detail)
- **THEN** 返回 Err(VerifyError::InvariantViolation { detail, source })

### Requirement: CompositeVerifier

CompositeVerifier SHALL 接受 Vec<Box<dyn Verifier>>，依次执行所有 verifier，收集所有错误。仅在全部通过时返回 Ok。

#### Scenario: 全部通过

- **WHEN** CompositeVerifier 包含 3 个 verifier 且全部返回 Ok
- **THEN** 返回 Ok(())

#### Scenario: 部分失败

- **WHEN** CompositeVerifier 包含 3 个 verifier 且 2 个返回 Err
- **THEN** 返回 Err(VerifyError::Composite(vec![2 个错误]))


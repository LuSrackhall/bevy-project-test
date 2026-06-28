# Scenario Test Harness — 工程规范

本规范定义 simulation crate 的场景测试框架使用约定。

## 架构

```
crates/simulation/src/scenario/
├── mod.rs              ← Scenario + ScenarioOutput + run()
├── verifier.rs         ← trait Verifier + VerifyError
├── verifiers/
│   ├── snapshot.rs     ← SnapshotVerifier（hash 比对）
│   ├── event.rs        ← EventVerifier（按 tick 校验事件）
│   ├── invariant.rs    ← InvariantVerifier（不变量检查）
│   └── composite.rs    ← CompositeVerifier（组合）
└── scenario_test.rs    ← 内置测试
```

## 写一个 Scenario 测试

```rust
use simulation::scenario::*;
use simulation::command::*;
use simulation::types::*;
use simulation::map;

#[test]
fn test_my_feature() {
    let result = Scenario {
        seed: 42,
        map_size: map::MapSize::Small,
        config: RunConfig::default(),
        commands: vec![
            GameCommand {
                tick: 5,
                player_id: 0,
                action: Action::MoveTo { unit: UnitId(1), target: FixedVec2::ZERO },
            },
        ],
        max_tick: 100,
        verifier: Box::new(CompositeVerifier(vec![
            Box::new(SnapshotVerifier::hash(EXPECTED_HASH)),
            Box::new(EventVerifier::new()
                .expect_spawned_at(1, |s| !s.is_empty())),
            Box::new(InvariantVerifier::new()
                .check(|world| {
                    // 不变量检查闭包
                    None // 返回 None 表示通过
                })),
        ])),
    }
    .run();

    assert!(result.is_ok(), "Scenario failed: {:?}", result.err());
}
```

## Verifier 使用规范

### SnapshotVerifier

用于确定性回归测试。调用 `hash_world_state` 比对最终状态哈希。

```rust
Box::new(SnapshotVerifier::hash(expected_hash))
```

**已知限制**：仅覆盖 `hash_world_state` 当前包含的组件。新增仿真组件时必须同步更新 `hash_world_state`（宪法 §10.2）。

### EventVerifier

按 tick 校验 SimulationEvents。Builder API 支持：

```rust
EventVerifier::new()
    .expect_spawned_at(tick, |events| /* bool */)
    .expect_captured_at(tick, |events| /* bool */)
    .expect_damage_at(tick, |events| /* bool */)
    .expect_destroyed_at(tick, |events| /* bool */)
    .expect_leveled_up_at(tick, |events| /* bool */)
```

### InvariantVerifier

检查仿真结束后的世界状态不变量。闭包接收 `&mut World`。

```rust
InvariantVerifier::new()
    .check(|world| {
        let mut q = world.query::<&Health>();
        for h in q.iter(world) {
            if h.current > h.max {
                return Some(format!("HP {} > max {}", h.current, h.max));
            }
        }
        None
    })
```

**注意**：闭包不得修改 World 状态（§17 Truth Ownership）。

### CompositeVerifier

组合多个 verifier，全部通过才返回 Ok。

```rust
Box::new(CompositeVerifier(vec![
    Box::new(verifier_a),
    Box::new(verifier_b),
]))
```

## RunConfig

控制仿真子系统开关。当前支持：

- `enable_ai: bool` — 是否执行 AI 决策阶段（默认 true）

测试中禁用 AI 以隔离玩家命令：

```rust
config: RunConfig { enable_ai: false }
```

## CI 集成

场景测试通过 `cargo test -p simulation` 运行，无需额外 CI 配置。

新增场景测试时遵循以下 checklist（宪法 §22.3）：

- [ ] 复杂度声明（§4.3）
- [ ] hash_world_state 覆盖更新（§10.2）— 如有新组件
- [ ] 确定性测试补充（§10.1）
- [ ] 不引入禁用类型（§1.4）
- [ ] 命令驱动，不直读外部输入（§2.5）

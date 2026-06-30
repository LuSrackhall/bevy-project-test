## Why

每 tick 4 个 combat 系统独立构建相似数据结构，造成 14 次全量 entity 扫描 + 6 次 SpatialHash 构建。Benchmark 数据：combat_engagement 在 3000 单位 53ms（O(n,k), k≈n 密集场景）。目标 100k 单位需要消除冗余扫描、控制 k 大小。

## What Changes

- 新增 TickCombatIndex Resource，tick 开始时构建一次，所有 combat 系统共享
- 按阵营索引 SpatialHash（faction_indices），跳过友方单位使 k 减半
- combat_engagement / melee_attack / archer_attack / arrow_movement 改为读取共享索引

## Capabilities

### New Capabilities
- `combat-shared-index`: TickCombatIndex 共享 Resource
- `combat-faction-spatial`: 按阵营 SpatialHash 索引

### Modified Capabilities
- `simulation-crate`: combat 系统重构
- `combat-fixes`: combat_engagement + melee + archer + arrow

## Impact

- `crates/simulation/src/combat/mod.rs` — 主要重构
- `crates/simulation/src/soldier/mod.rs` — build_soldier_index 调用变为一次
- `crates/simulation/src/lib.rs` — run_tick 开始时构建 TickCombatIndex
- 107 现有测试 + golden_test 必须通过

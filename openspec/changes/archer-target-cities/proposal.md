## Why

弓兵当前只以敌方兵种为攻击目标。射程内无敌方兵种时，弓兵不发射箭矢，恢复 Moving 状态。城池作为大型战略目标，应在兵种优先的前提下成为弓兵的次要攻击目标。

## What Changes

- `archer_attack_system`: 兵种搜索无结果时，追加城池搜索。射程内最近敌方城池作为备选目标
- 箭矢对城池的 1/200 低伤害保持不变（已有 `arrow_movement_system` 城池碰撞逻辑处理）
- 城池伤害 1/200 是刻意设计：弓兵攻城刮痧，主力攻城手段为自杀式步兵

## Capabilities

### New Capabilities

无。此为现有 `archer-directional-arrow` spec 的行为扩展。

### Modified Capabilities

- `archer-directional-arrow`: 战斗状态互斥 — 无兵种目标时改为搜索城池，而非直接恢复 Moving。新增城池目标场景。

## Impact

- `crates/simulation/src/combat/mod.rs`: `archer_attack_system` 目标搜索逻辑扩展（约 15 行新增代码）

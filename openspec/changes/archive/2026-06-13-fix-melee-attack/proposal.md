## Why

非弓兵单位（民兵、步兵、骑兵）无法稳定攻击敌方单位。弓兵正常（因为使用独立的目标扫描机制），但近战单位的攻击行为间歇性失效。

**根本原因分析：**

1. **到达处理清除目标（主因）**：移动系统中，当单位距离目标 < 5 时触发"到达"，清除 `Movement.target` 并将状态重置为 `Moving`。但碰撞体积让单位无法真正到达距离 5 以内（被 overlap 推开到 ~12），导致目标在到达与被推开之间反复丢失。近战系统（Phase 8）读取目标时为 `None`，无法攻击。

2. **骑兵不被设置目标（次因）**：`combat_engagement_system` 中 `if !is_cav` 守卫跳过了骑兵的 `Movement` 更新，导致骑兵永远没有自动攻击目标。

3. **近战系统依赖 `Movement.target`（架构问题）**：弓兵的攻击系统直接扫描所有敌人位置，不依赖 `Movement.target`。但近战系统依赖 `Movement.target` 来找到攻击目标，而这个目标会被移动系统的到达处理清除。

## What Changes

- **移动系统**：将弓兵的"在射程内保持目标"逻辑扩展到所有兵种。当 Fighting 状态的单位在攻击范围内时，跳过移动、保留目标。
- **移动系统**：Fighting 状态且有目标的单位，禁用到达处理（不清除目标、不改变状态）。
- **索敌系统**：移除 `if !is_cav` 守卫，骑兵也正确设置 `Movement.target`。

## Capabilities

### Modified Capabilities
- `soldier-movement`: 近战单位在攻击范围内保持目标、Fighting 状态禁用到达
- `combat-engagement`: 骑兵正确接收自动攻击目标

## Impact

- `simulation/src/soldier/mod.rs` — 移动系统（到达处理 + 射程内保持）
- `simulation/src/combat/mod.rs` — 索敌系统（骑兵目标设置）

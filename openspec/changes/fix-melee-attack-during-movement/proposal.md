## Why

近战单位（民兵、步兵、骑兵）在移动过程中无法自动攻击范围内的敌人。弓兵正常（因为使用独立的目标扫描机制），但近战单位的攻击行为完全失效。

**根因分析：**

1. **MoveTo 的 force_move 参数错误（主因）**：`consume_commands_system` 中，`MoveTo` 和 `ForceMove` 都传了 `force_move: true`。这导致所有移动命令都被当作"强制移动"，`combat_engagement_system` 因 `if sd.force_move { continue; }` 跳过了索敌，单位永远不会自动设目标。

2. **近战系统依赖 Fighting 状态**：`melee_attack_system` 只对 `SoldierState::Fighting` 的单位生效，但 `Fighting` 状态只有通过索敌系统或手动 Attack 命令才能设置。移动中的单位状态为 `Moving`，即使敌人在攻击范围内也不会进入攻击逻辑。

3. **到达处理清除目标**：当单位到达目标位置（距离 < 5）时，`Movement.target` 被清除，导致近战系统无法找到攻击目标。

## What Changes

### 1. 修复 MoveTo 的 force_move 参数
- `MoveTo` → `apply_movement(world, unit, target, false)`（允许移动中自动攻击）
- `ForceMove` → `apply_movement(world, unit, target, true)`（强制移动不攻击）

### 2. 移动中攻击机制
- 近战系统不再依赖 `SoldierState::Fighting` 状态
- 每个 tick，对每个有 `Movement.target` 的近战单位，检查目标是否在攻击范围内
- 如果在范围内且冷却归零 → 攻击（不影响移动命令）
- 攻击后继续移动（不改变状态、不清除目标、不重置冷却以外的组件）

### 3. ForceMove 抑制自动攻击
- 当 `force_move == true` 时，跳过移动中攻击逻辑
- 例外：骑兵的攻击不影响移动，因此骑兵在 ForceMove 时仍可攻击

### 4. 骑兵特殊处理
- 骑兵在任何移动状态下都可攻击（MoveTo 和 ForceMove）
- 骑兵攻击不停顿、不减速

## Capabilities

### Modified Capabilities
- `consume_commands-system`: 修复 MoveTo 的 force_move 参数
- `melee-attack-system`: 支持移动中攻击（不依赖 Fighting 状态）
- `combat-engagement-system`: 保持现有逻辑（force_move 跳过索敌）

## Impact

- `simulation/src/soldier/mod.rs` — consume_commands_system（force_move 修复）
- `simulation/src/combat/mod.rs` — melee_attack_system（移动中攻击逻辑）

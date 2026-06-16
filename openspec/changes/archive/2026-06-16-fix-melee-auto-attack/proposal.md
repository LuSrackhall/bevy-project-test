## Why

近战单位（民兵、步兵、骑兵）在站立不动或移动过程中，即使敌人在攻击范围内，也无法自动攻击。

**根因：** `melee_attack_system` 依赖 `Movement.target` 来确定攻击目标。但 `Movement.target` 只在以下情况被设置：
- `combat_engagement_system` 设置（需要 `SeekStance.active == true` 且 `seek_range > 0`）
- 手动 `Attack` 命令

站立不动且没有活跃 SeekStance 的单位，`Movement.target` 永远是 `None`，近战系统永远跳过。

**对比弓兵：** `archer_attack_system` 直接扫描所有敌人位置，不依赖 `Movement.target`，所以弓兵能正常攻击。

## What Changes

### 1. 近战系统改为直接扫描
- `melee_attack_system` 不再依赖 `Movement.target`
- 改为像弓兵一样，每 tick 扫描攻击范围内的所有敌人
- 选择最近的敌人作为攻击目标
- 站立不动、移动中、任何状态下都能自动攻击

### 2. 朝向影响攻速（新增）
- 攻速因子 = `1 + 0.3 × cos(朝向偏差角)`
- 正面对敌(0°): 攻速 ×1.3（快30%）
- 侧面对敌(90°): 攻速 ×1.0（正常）
- 背面对敌(180°): 攻速 ×0.7（慢30%）
- 使用定点数 cos 近似计算

### 3. ForceMove 抑制保持不变
- `force_move == true` 且非骑兵 → 跳过攻击
- 骑兵在任何移动状态下都可攻击

## Capabilities

### Modified Capabilities
- `melee-attack-system`: 从依赖 Movement.target 改为直接扫描敌人
- `attack-speed`: 新增朝向对攻速的线性影响

## Impact

- `simulation/src/combat/mod.rs` — melee_attack_system 重写目标选择逻辑
- `simulation/src/facing.rs` — 新增 `cos_approx` 函数（定点数 cos 近似）

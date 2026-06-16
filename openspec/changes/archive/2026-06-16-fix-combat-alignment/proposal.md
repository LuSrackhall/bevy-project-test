## Why

经审计发现代码实现与规范存在 3 处冲突，需要修复代码以匹配 BEHAVIOR 规范。

## What Changes

### 1. 近战前摇恢复
- `melee_attack_system` 不再直接造成伤害
- 改为创建 `AttackWindup` 条目（非骑兵 3 ticks，骑兵立即攻击）
- `attack_windup_system` 处理前摇完成后的伤害逻辑
- 前摇期间单位不移动（但朝向仍调整）

### 2. 盾牌手动格挡移速修正
- 当前：`speed -= speed_penalty`（80-15=65）
- 修改：手动格挡时移速直接设为 15

### 3. 非正面伤害跳过被动格挡
- 当前：手动格挡时，非正面伤害仍走被动格挡判定（40%）
- 修改：手动格挡时，非正面伤害直接扣士兵 HP，不经过被动格挡

### 4. 文档同步
- 更新 MAIN 设计文档，加入近战自动扫描、朝向攻速、MoveTo/ForceMove、箭矢散布等新机制

## Capabilities

### Modified Capabilities
- `melee-attack-system`: 恢复前摇机制
- `shield-manual-block`: 修正移速和非正面伤害处理

## Impact

- `simulation/src/combat/mod.rs` — melee_attack_system、try_passive_block
- `simulation/src/soldier/mod.rs` — 移速计算
- `docs/superpowers/specs/2026-06-05-rts-game-design.md` — 文档同步

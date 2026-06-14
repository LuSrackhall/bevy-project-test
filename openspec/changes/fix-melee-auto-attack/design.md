## Context

近战系统当前依赖 `Movement.target` 来确定攻击目标，这导致站立不动的单位无法自动攻击。需要改为直接扫描模式。

## Goals / Non-Goals

**Goals:**
- 近战单位在任何状态下都能自动攻击范围内敌人
- 朝向影响攻速（线性关系）
- 保持确定性（定点数计算）

**Non-Goals:**
- 不改变弓兵行为
- 不改变索敌系统的触发条件
- 不引入新的状态枚举

## Decisions

### D1: 近战目标选择 — 直接扫描

不再依赖 `Movement.target`，改为：
1. 收集所有敌人士兵位置（与弓兵相同的 HashMap 方式）
2. 对每个近战单位，遍历敌人列表
3. 筛选距离 ≤ 攻击范围的敌人
4. 选择最近的敌人作为攻击目标
5. 如果有多个等距敌人，选择 UnitId 最小的（确定性）

### D2: 朝向攻速因子 — 线性 cos 关系

```
攻速因子 = 1 + 0.3 × cos(朝向偏差角)
```

- 朝向偏差角 = `angle_distance(attacker_facing, angle_to_target)`
- cos 使用多项式近似（定点数）：`cos(x) ≈ 1 - x²/2 + x⁴/24`
- 攻速因子范围：[0.7, 1.3]
- 应用方式：`effective_interval = base_interval / factor`
  - 正面：interval / 1.3 ≈ 0.77 × base（更快）
  - 侧面：interval / 1.0 = base（正常）
  - 背面：interval / 0.7 ≈ 1.43 × base（更慢）

### D3: 骑兵特殊处理

骑兵的攻击无前摇（`cavalry_no_windup = true`），朝向攻速因子不适用于骑兵的即时攻击。但骑兵的攻击冷却仍受朝向影响。

## Risks / Trade-offs

- **性能**：每 tick 每个近战单位都要遍历敌人列表。当前单位数量较少（< 100），性能影响可忽略。
- **确定性**：使用 UnitId 最小值作为 tie-breaker，保证确定性。

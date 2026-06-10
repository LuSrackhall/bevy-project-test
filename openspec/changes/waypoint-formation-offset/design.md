## Context

分离力机制（unit-separation）对 waypoint 汇聚场景无效：所有兵都被同一个点吸引，排斥力不足以对抗引力，且到达后分离力不再执行。

## Goals / Non-Goals

**Goals:**
- waypoint 移动时每个兵自动分散到唯一位置（32×32 网格，8 单位间距）
- 城池目标也应用偏移（弓兵/近战攻城时自然分散）
- 敌方兵种目标不应用偏移（保留分离力，近战自然成环）

**Non-Goals:**
- 不替代分离力（敌方兵种目标仍用分离力）

## Decisions

### D1: personal_offset 公式

```rust
fn personal_offset(unit_id: UnitId, spacing: i32) -> (i32, i32) {
    let n = (unit_id.0 % 1024) as i32;
    let grid = 32;
    let x = (n % grid - grid / 2) * spacing;  // [-128, +120]
    let y = (n / grid - grid / 2) * spacing;
    (x, y)
}
```

32×32 网格 = 1024 个独立位置，间距 8 单位。纯整数，O(1)。

### D2: 偏移应用判定

```rust
// waypoint 或 城池目标 → 应用偏移
// 敌方兵种目标 → 不偏移，走分离力
let apply_offset = mov.waypoint.is_some() || is_city_target;
```

### D3: 偏移在 target_pos 计算时叠加

与现有 waypoint 和 city target 路径一致，偏移后的 effective_target 进入正常移动计算流程。

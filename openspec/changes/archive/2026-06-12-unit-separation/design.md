## Context

`soldier_movement_system` 每 Tick 为每个士兵计算朝向目标的移动向量，直接写入新位置。整个过程无碰撞检测。

## Goals / Non-Goals

**Goals:**
- 士兵之间保持最小间距，防止视觉重叠
- O(n) 性能，支持万人同屏
- 纯 Fixed 定点运算，完全确定

**Non-Goals:**
- 不实现寻路绕障
- 不处理兵种与城池的物理碰撞
- 不影响攻击判定距离

## Decisions

### D1: 空间哈希网格

**选择**: HashMap 网格，cell_size = separation_radius × 2。

**理由**: O(n) 构建 + O(1) 邻居查询（每兵只查 9 格）。万人场景：~125×125 网格，每格 ~0.64 兵，每兵 ~6 次检查，总计 ~60K 次 — 远低于 O(n²) 的 50M 次。

### D2: 分离力后置（调整最终位置）

**选择**: 先计算正常移动目标位置，再对目标位置施加分离偏移。

**理由**: 分离是在"我想到达的位置"与"别人在哪里"之间做微调，不影响移动速度和方向计算。权重默认 0.4（40% 的分离强度），足够分离而不导致抖动。

### D3: 分离公式

```
对每个邻居（距离 < separation_radius）:
  repulsion = (my_target - neighbor_pos) / max(distance, 1)
  total_force += repulsion

final_pos = target_pos + total_force × separation_weight
```

分母用 `max(distance, 1)` 防止零除。越近推力越大，自然分散。

### D4: 配置文件

新增 `content/combat.ron` 配置段：
```ron
unit_separation: (
    separation_radius: 8,
    separation_weight: 0.40,
)
```

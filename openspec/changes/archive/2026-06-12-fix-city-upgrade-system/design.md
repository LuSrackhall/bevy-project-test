## Context

`city_interaction_system` 中友方士兵进城升级分支（`soldier/mod.rs:349-365`）只更新了 `level`、`health_max`、`health_current` 三个字段，遗漏了 `max_population` 和 `CityRadius`。

升级成本公式 `req = max_hp × level_up_cost_multiplier` 中 multiplier 为 100.0，与 gain_ratio (0.30) 不匹配，导致 334 个士兵才能升一级。

## Goals / Non-Goals

**Goals:**
- 升级公式改为 `req = max_hp × cost_multiplier × level`（`cost_multiplier = 1.0`），难度随等级线性增长
- 升级时 `max_population` 按 `level × base_population_per_level` 更新
- 升级时 `CityRadius` 按 `visual_radius_base + level × visual_radius_per_level` 更新
- 公式天然适配任意等级上限（不含硬编码天花板）

**Non-Goals:**
- 不改变士兵进城后的人口转移逻辑（问题 4，待后续讨论）

## Decisions

### D1: 升级公式改为 req = max_hp × cost_multiplier × level, cost_multiplier = 1.0

**选择**: 将 `level_up_cost_multiplier` 从 100.0 改为 1.0，req 公式乘上 `× level`。

**理由**: 旧公式 `req = max_hp × cost` 与 `gain = max_hp × gain_ratio` 的分母（max_hp）互相抵消，导致所有等级所需士兵数完全相同。新公式 `req = max_hp × cost × level` 使需求随等级线性增长：

```
soldiers = cost × level / gain_ratio = 1.0 × level / 0.30 ≈ 3.33 × level
```

| Lv.变化 | 所需士兵 | | Lv.变化 | 所需士兵 |
|---------|---------|---|---------|---------|
| 1→2     | ~4      | | 10→11   | ~34     |
| 2→3     | ~7      | | 20→21   | ~67     |
| 3→4     | ~10     | | 50→51   | ~167    |
| 5→6     | ~17     | | 100→101 | ~334    |

公式无硬编码上限，支持任意等级。高等级时所需士兵自然增加，形成软性制约。

**替代方案**: 用 `level²` 做二次增长 → 太陡，Lv.10 就要 334 兵。用 `√level` → Lv.100 只要 34 兵，太容易。线性增长是最佳平衡。

### D2: max_population 更新公式

**选择**: 升级时 `max_population = level × base_population_per_level`，不再使用随机额外值。

**理由**: 随机额外值仅在生成时用于地图多样性。升级后直接使用基础公式，确定性结果，也防止升级后 max_population 被随机压低。

### D3: CityRadius 使用与 generate_map 和 city_capture_check 一致的公式

**选择**: `CityRadius = visual_radius_base + level × visual_radius_per_level`。

**理由**: 与已有的两处 CityRadius 计算保持一致。

## Risks / Trade-offs

- **[Lv.100 需 334 兵]**：线性公式在极端等级下自然变难，形成软上限。如需要更高等级天花板，可调低 `cost_multiplier` 或提高 `gain_ratio`，无需改代码。
- **[中途调整配置影响已有存档]**：`level_up_cost_multiplier` 的变更仅影响未来的升级，已累计的 `level_exp` 不变。

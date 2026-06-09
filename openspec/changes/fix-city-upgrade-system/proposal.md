## Why

城池升级系统存在三个实现问题，导致升级功能在正常游戏中几乎不可用：

1. **升级成本过高**：`level_up_cost_multiplier: 100.0` 导致需要 334 个士兵才能升一级。且当前公式 `req = max_hp × cost_multiplier` 与 gain 公式 `gain = max_hp × gain_ratio` 的分母（max_hp）互相抵消，导致所有等级恒定 10 兵/级。新公式 `req = max_hp × cost_multiplier × level` 使升级难度随等级线性增长，同时 `cost_multiplier` 降至 `1.0`。公式天然支持任意等级上限。
2. **max_population 不随等级增长**：升级后人口上限不变，高等级城池无法容纳更多人口
3. **CityRadius 不随升级更新**：城池视觉/碰撞半径与实际等级不同步

## What Changes

- `content/cities.ron`: `level_up_cost_multiplier` 从 `100.0` 改为 `1.0`
- `city_interaction_system`: 升级经验公式改为 `req = max_hp × cost_multiplier × level`，升级时同步更新 `max_population` 和 `CityRadius`
- 公式天然支持更高等级上限（Lv.20→21 约需 67 兵，Lv.50→51 约需 167 兵）

## Capabilities

### New Capabilities

- `city-upgrade-fixes`: 城池升级时同步更新 max_population 和 CityRadius

### Modified Capabilities

- `simulation-crate`: 城池升级行为变更 — 升级成本调整，升级时额外更新人口上限和半径

## Impact

- `crates/simulation/src/soldier/mod.rs`: `city_interaction_system` 升级分支新增 max_population 更新和 CityRadius 更新
- `content/cities.ron`: `level_up_cost_multiplier` 值变更

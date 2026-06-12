## Why

城池底部详情面板中经验栏始终显示 `"?/?"`，因为 `update_bottom_panel` 从未写入 `exp_text` 文本实体。玩家看不到城池升级进度。

## What Changes

- `update_bottom_panel` 新增 `exp_text` 更新，显示 `"经验 {level_exp}/{required_exp}"`，required_exp 根据 `city.health_max × level_up_cost_multiplier` 计算

## Capabilities

### New Capabilities

无。此为纯 bug 修复。

### Modified Capabilities

无。现有 `render-view-crate` spec 已要求底部面板显示经验。

## Impact

- `crates/render_view/src/ui/hud.rs`: `update_bottom_panel` 新增 exp_text 更新逻辑（约 5 行代码）
- 需从 sim world 读取 `CityGlobalConfig` 获取 `level_up_cost_multiplier`

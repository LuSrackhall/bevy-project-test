## Context

`update_bottom_panel` 在 `crates/render_view/src/ui/hud.rs` 中负责刷新选中城池的底部面板。已有代码更新了 city_info、hp_text、hp_fill、pop_detail、spawn_type_text，但 `exp_text` 在 `setup_hud` 中创建后从未被写入。

城池经验数据来自 `CityComponent::level_exp: u64`，升级阈值 = `health_max × CityGlobalConfig::level_up_cost_multiplier`（参见 `city_interaction_system`）。

## Goals / Non-Goals

**Goals:**
- 底部面板正确显示城池经验 `"经验 {当前}/{所需}"`

**Non-Goals:**
- 不改变经验计算逻辑
- 不改动城池升级机制

## Decisions

### D1: 与现有代码风格一致

在 `update_bottom_panel` 末尾（`spawn_type_text` 更新之后）插入 exp_text 更新块，结构与已有字段更新完全一致。

## Risks / Trade-offs

无风险。纯 UI 展示修复，不影响仿真逻辑。

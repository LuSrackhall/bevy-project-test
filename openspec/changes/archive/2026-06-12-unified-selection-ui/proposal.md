## Why

当前选中系统混乱：城池用 `SelectedCity`、兵种用 `selected_unit_ids` 两套独立资源，城池面板和兵种面板在不同位置，可能同时显示。框选会意外选中城池实体。设计上城池和兵种应互斥选中，框选只选兵种。

## What Changes

- **选中统一**：移除 `SelectedCity`，城池选中也走 `selected_unit_ids`（或一个独立的 `Option<CityEntity>`）。城池与兵种互斥
- **点击优先级**：先查城池 → 再查兵种。命中城池清空兵种选中
- **框选过滤**：`drag_select_system` 查询加 `SoldierMarker`，忽略城池
- **面板统一**：士兵和城池信息在同一个 30% 面板位置展示，根据选中类型切换内容

## Capabilities

### Modified Capabilities

- `render-view-crate`: HUD 简化布局 + 选中逻辑统一
- `city-interaction`: 移除 `SelectedCity` 资源，城池选中由统一选中系统管理

## Impact

- `crates/render_view/src/ui/hud.rs`: 面板合并，移除城池专属面板区，移除 `SelectedCity`
- `crates/render_view/src/ui/mod.rs`: 移除 `SelectedCity` 相关注册
- `crates/render_view/src/selection.rs`: `drag_select_system` + `SoldierMarker` 过滤；`selection_click_system` + 城池优先

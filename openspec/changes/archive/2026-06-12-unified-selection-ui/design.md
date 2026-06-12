## Context

当前两套选中系统同时运行，面板在三处不同位置展示。框选查询无 `SoldierMarker` 过滤，意外包含城池。

## Goals / Non-Goals

**Goals:**
- 城池和兵种互斥选中
- 框选只选兵种
- 面板统一在一个位置
- 代码简化（移除 `SelectedCity` 资源和 `city_click_system`）

## Decisions

### D1: 选中互斥

```rust
SelectionState { selected_units: Vec<UnitId>, selected_city: Option<UnitId> }
// 或更简单：直接用一个 enum

// 更新逻辑：
// 选中城池 → selected_city = Some(id), selected_units = []
// 选中兵种 → selected_units = [...], selected_city = None
```

### D2: 点击优先级

```
click → 先查 CityMarker → 命中 → 选中城池
      → 没命中 → 查 SoldierMarker → 命中 → 选中兵种
      → 都没命中 → 清空全部选中
```

### D3: 框选过滤

```
drag_select: query + With<SoldierMarker> → 城池不可框选
```

### D4: 面板统二为二区

```
左 30%: 选中信息（城池/兵种共用，根据选中类型切换内容）
右 70%: 命令卡片 + 图鉴
```

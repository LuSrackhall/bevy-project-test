## Context

当前布局为三区，兵种和城池信息在不同列。需将二者叠在同一位置。

## Goals / Non-Goals

**Goals:**
- 左 30% 统一承载城池+兵种信息
- 右 70% 承载命令卡片+图鉴
- 城池/兵种面板互斥显示

## Decisions

### D1: 面板叠加在同一 Node

城池面板和士兵面板都是左 30% 容器的子节点。通过 `Display::Flex/None` 控制哪个可见。

```rust
// Left 30% container
.with_children(|p| {
    // Soldier panel (visible by default when no city)
    p.spawn(soldier_panel).insert(PanelVisibility::Soldier);
    // City panel (hidden by default)
    p.spawn(city_panel).insert(PanelVisibility::City);
})

// Update: 查 PanelVisibility 标记，切换 Display
```

### D2: 布局简化为二区

```rust
// Left 30%: info panel
p.spawn(Node { width: 30%, ... }) → soldier + city panels
// Right 70%: command card + compendium
p.spawn(Node { width: 70%, flex_dir: Column, ... }) → command card + compendium
```

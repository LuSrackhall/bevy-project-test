## Context

`render_view/src/unit_info_bar.rs` 的 `create_bar` 函数中，`let vis = if visible { Visibility::Inherited } else { Visibility::Hidden }` 被同时用于根实体和所有子实体（文字、背景条、填充条）。

`update_bar` 中只通过 `root_xform_vis` 查询修改根实体的 `Visibility`，子实体的 `Visibility` 不会被更新。当子实体被创建为 `Hidden` 时，即使父级变为 `Inherited`，子实体仍然隐藏。

## Goals / Non-Goals

**Goals:**
- 子实体始终使用 `Visibility::Inherited`，由父级统一控制可见性
- 城池和士兵的血条在任何显示模式下都能正确显示/隐藏
- 切换显示模式后，已有单位的血条立即生效

**Non-Goals:**
- 不修改 `update_bar` 的可见性逻辑（已正确）
- 不修改显示模式的判断逻辑
- 不修改组件架构

## Decisions

### 1. 子实体始终使用 `Visibility::Inherited`

**选择：** `create_bar` 中引入 `root_vis` 变量用于根实体，子实体硬编码 `Visibility::Inherited`。

**原理：** `Visibility::Hidden` 是显式隐藏，不继承父级。`Visibility::Inherited` 跟随父级状态。子实体应始终跟随父级，由根实体统一控制。

**替代方案：** 在 `update_bar` 中遍历所有子实体修改 `Visibility` → 改动过大，且 `Children` 查询在 Bevy 0.18 中有兼容性问题。

## Risks / Trade-offs

- **风险：** 无。这是纯逻辑修正，恢复了设计意图。

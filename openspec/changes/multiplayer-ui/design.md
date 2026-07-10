## Context

当前联机主菜单中 `NetworkPlayerCount` 和 `NetworkPlayerId` 按钮为静态显示。详见 `brainstorm-spec.md`。变更范围：仅 `crates/render_view/src/ui/menu.rs`。

## Goals / Non-Goals

**Goals:**
- 两个按钮可点击循环值，Text 同步更新
- 开始按钮 Query 跨兄弟实体获取配置值

**Non-Goals:**
- 同 brainstorm-spec.md

## Decisions

详见 brainstorm-spec.md 的 Decisions 表。核心：

1. **内联 observer**：匹配 menu.rs 现有 4 个模式
2. **跨实体 clamp**：Count observer 增加 `Query<(&mut NetworkPlayerId, &Children)>` 写入 ID 组件和子 Text
3. **开始按钮修复**：三个独立 Query，每项值独立 `iter().next()` 配合 `unwrap_or(default)`

## Risks / Trade-offs

详见 brainstorm-spec.md 的 Risks 表。

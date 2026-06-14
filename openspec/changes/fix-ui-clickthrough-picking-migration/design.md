## Context

当前项目使用 `Input<MouseButton>` + 手动 `UiFocusBlocker` 资源处理 UI 点击穿透。`UiFocusBlocker` 每帧重置为 `false`，各 UI 系统在检测到 `Interaction::Pressed` 时设置 `blocked = true`，`selection_click_system` 检查该标志后决定是否处理游戏世界点击。

问题：`soldier_type_button_system`、`toolbar_button_system` 等系统未设置 `blocked`，导致点击这些按钮时穿透到游戏世界。`command_issue_system`（右键命令）完全没有 blocker 检查。

Bevy 0.18 的 `ui_focus_system` 在 `PreUpdate` 阶段自动维护所有 UI 节点的 `Interaction` 组件状态。当按钮被按下时，`Interaction` 变为 `Pressed`；透明容器只有 `Hovered`（被按钮阻挡）。这提供了区分"点击 UI 按钮"和"点击游戏世界"的天然机制。

## Goals / Non-Goals

**Goals:**

- 用 Bevy 原生 Picking 系统替代手动 `UiFocusBlocker`，消除 UI 点击穿透 bug
- 为 `selection_click_system` 和 `command_issue_system` 提供统一的 UI 穿透保护
- 验证 Observer 机制在当前项目中的可行性，为后续 `bevy_ui_widgets` 迁移积累实证数据
- 删除所有手动 blocker 相关代码，简化架构

**Non-Goals:**

- 不迁移按钮到 `bevy_ui_widgets`（Phase 2）
- 不改变按钮的 `Interaction` 查询模式（Phase 2）
- 不处理拖拽框选的穿透（后续单独处理）
- 不修改 Seek Panel 的交互逻辑（仅删除其 blocker 参数）

## Decisions

### Decision 1: 用 Interaction::Pressed 替代 UiFocusBlocker

**选择**: 在 `selection_click_system` 和 `command_issue_system` 中查询 `Query<&Interaction>`，检查是否有任何 UI 元素处于 `Interaction::Pressed` 状态。

**备选方案**:
- A) 为每个 UI 系统手动补 blocker → 维护成本高，容易遗漏
- B) 查询 `Res<HoverMap>` 判断光标是否在 UI 上 → 无法区分"空白处"和"游戏实体上"（两者 HoverMap 均为空）
- C) 把 selection 系统也改为 Pointer 事件 → 需要游戏实体 pickable，改动大

**理由**: `Interaction::Pressed` 只在按钮被实际按下时为 true。透明容器只有 `Interaction::Hovered`（被按钮阻挡），不会误判。`ui_focus_system` 在 `PreUpdate` 运行，`selection_click_system` 在 `Update` 运行，时序正确。不需要修改透明容器的 Pickable 设置。

**HoverMap 方案失败原因**: HoverMap 在光标位于空白处和游戏实体上时均为空，无法区分两种情况。即使给透明容器添加 `Pickable::IGNORE`，仍然无法解决"点击空白处应清除选中"和"点击游戏实体应选中"的矛盾。

### Decision 1b: 透明容器添加 Pickable::IGNORE

**选择**: 在 HUD 布局中的透明容器节点（根节点、spacer、底部区域容器、左右面板容器）上添加 `Pickable::IGNORE`。

**问题**: Bevy 的 Picking 命中检测基于布局边界（`ComputedNode::contains_point`），不检查视觉属性。没有 `Pickable` 组件的节点默认 `should_block_lower: true`，即使节点是透明的也会阻挡下层。这导致 HoverMap 在光标位于透明容器上时非空，`is_cursor_over_ui` 误判为"光标在 UI 上"。

**备选方案**:
- A) 在 `is_cursor_over_ui` 中检查 HoverMap 条目的具体类型 → 需要额外查询，复杂度高
- B) 重构 HUD 布局，消除透明容器 → 改动大，影响布局结构
- C) 透明容器添加 `Pickable::IGNORE` → 最简单，不改变布局

**理由**: `Pickable::IGNORE` 告诉 Picking 系统忽略该节点，不阻挡下层。透明容器（根节点、spacer、底部区域）本身不需要接收点击事件，它们的子节点（按钮、面板）仍保持默认 Pickable 行为。这是 Bevy 推荐的处理透明容器的方式。

### Decision 2: Observer 验证采用全局 Observer + Pointer<Click>

**选择**: 用 `app.add_observer(|ev: On<Pointer<Click>>| {...})` 监听所有点击事件，在 Observer 内通过 Query 判断是否命中目标按钮。

**备选方案**:
- A) 实体级 Observer `.observe(...)` → 需要在 spawn 时注册，改动大
- B) 自定义事件桥接 → 增加不必要的间接层
- C) 仅验证 Observer 机制，不涉及 Pointer → 验证不充分

**理由**: 全局 Observer 最简单，不需要修改 spawn 代码。`Pointer<Click>` 在 UI 按钮上会正确触发（已从源码确认），`ev.entity()` 返回被点击的按钮实体。Observer 可访问 `ResMut`、`Query` 等标准系统参数（已从源码确认）。

### Decision 1b: 透明容器添加 Pickable::IGNORE（预防性措施）

**选择**: 在 HUD 布局中的透明容器节点（根节点、spacer、底部区域容器、左右面板容器）上添加 `Pickable::IGNORE`。

**当前状态**: 由于改用 `Interaction::Pressed` 方案，透明容器不再影响穿透检测。`Pickable::IGNORE` 作为预防性措施保留，确保 Picking 系统的行为符合预期（透明容器不阻挡下层），为未来可能的 Picking 相关功能打下基础。

### Decision 2: Observer 验证采用全局 Observer + Pointer<Click>

**选择**: 用 `app.add_observer(|ev: On<Pointer<Click>>| {...})` 监听所有点击事件，在 Observer 内通过 Query 判断是否命中目标按钮。

**理由**: 全局 Observer 最简单，不需要修改 spawn 代码。`Pointer<Click>` 在 UI 按钮上会正确触发（已从源码确认），`ev.entity()` 返回被点击的按钮实体。Observer 可访问 `ResMut`、`Query` 等标准系统参数（已从源码确认）。

### Decision 3: Phase 1a 和 Phase 1b 作为独立验证步骤

**选择**: Phase 1a 验证 Observer 机制（menu 按钮），Phase 1b 验证穿透修复（Interaction::Pressed 替代 blocker）。

**理由**: 两个验证目标独立。Phase 1a 不涉及穿透修复，Phase 1b 不涉及 Observer。分开验证降低单步风险，失败时可独立回滚。Phase 1a 的 Observer 验证为 Phase 2 的 bevy_ui_widgets 迁移提供实证数据。

## Risks / Trade-offs

**[Interaction::Pressed 时序依赖]** → 概率低。`ui_focus_system` 在 `PreUpdate` 运行，`selection_click_system` 在 `Update` 运行，时序正确。如果未来 Bevy 改变 `ui_focus_system` 的执行阶段，可能需要调整。

**[透明容器误判]** → 已解决。透明容器只有 `Interaction::Hovered`（被按钮阻挡），不会产生 `Interaction::Pressed`。`Pickable::IGNORE` 作为额外保障。

**[Observer 闭包调试困难]** → 可接受。Phase 1a 仅用于验证，Observer 内业务逻辑抽离为独立函数，闭包只做事件转发。

**[bevy_ui_widgets 仍为 experimental]** → 不影响。Phase 1a 使用原生 Observer + `Pointer<Click>`，不引入 `bevy_ui_widgets`。Phase 2 是否引入取决于 Phase 1a 的实证结果。

**[删除 blocker 后 seek panel 行为变化]** → 无影响。seek panel 系统的 blocker 参数仅用于防止穿透，删除后 `Interaction::Pressed` 检查在 selection 层面统一处理，seek panel 功能不受影响。

## Context

项目使用 `Input<MouseButton>` + 手动 `UiFocusBlocker` 资源处理 UI 点击穿透。`UiFocusBlocker` 每帧重置为 `false`，各 UI 系统在检测到 `Interaction::Pressed` 时设置 `blocked = true`，`selection_click_system` 检查该标志后决定是否处理游戏世界点击。

问题：`soldier_type_button_system`、`toolbar_button_system` 等系统未设置 `blocked`，导致点击这些按钮时穿透到游戏世界。`command_issue_system`（右键命令）完全没有 blocker 检查。

Bevy 0.18 的 Picking 系统已默认启用，UI 节点天然参与 picking 并阻止下层被 hover。`HoverMap` 资源由 Picking 系统在 `PreUpdate` 阶段自动维护，记录每个指针当前 hover 的实体。

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

### Decision 1: 用 HoverMap 替代 UiFocusBlocker

**选择**: 在 `selection_click_system` 和 `command_issue_system` 中查询 `Res<HoverMap>`，通过 `is_cursor_over_ui` 函数判断光标是否在 UI 上。

**实现**: `is_cursor_over_ui` 遍历 `HoverMap` 中鼠标指针下的所有实体，检查是否有任何实体带有 `Node` 组件（UI 节点标识）。

**备选方案**:
- A) 为每个 UI 系统手动补 blocker → 维护成本高，容易遗漏
- B) 查询 `Query<&Interaction>` 判断是否有 UI 被 pressed → `MenuButton`/`MenuItem` 不自动插入 `Interaction` 组件
- C) 查询 `Query<&PickingInteraction>` → `PickingInteraction` 不自动插入到未被 hover 的实体，首次 hover 时通过 commands 延迟插入，时序不可靠
- D) 查询 `Query<&Pressed>` → `Pressed` 只在 `bevy_ui_widgets::Button` 上存在，`MenuButton`/`MenuItem` 不使用

**理由**: `HoverMap` 由 Picking 系统自动维护，覆盖所有 UI 节点。透明容器添加 `Pickable::IGNORE` 后不阻挡下层，`HoverMap` 只包含实际的 UI 元素（按钮、面板等）。这是最通用的方案，不依赖特定组件类型。

### Decision 2: 透明容器添加 Pickable::IGNORE

**选择**: 在 HUD 布局中的透明容器节点（根节点、spacer、底部区域容器、左右面板容器、SeekPanelRoot）上添加 `Pickable::IGNORE`。

**问题**: Bevy 的 Picking 命中检测基于布局边界（`ComputedNode::contains_point`），不检查视觉属性。没有 `Pickable` 组件的节点默认 `should_block_lower: true`，即使节点是透明的也会阻挡下层。

**理由**: `Pickable::IGNORE` 告诉 Picking 系统忽略该节点，不阻挡下层。透明容器本身不需要接收点击事件，它们的子节点（按钮、面板）仍保持默认 Pickable 行为。

### Decision 3: Observer 验证采用 Pointer<Press>

**选择**: 用 `app.add_observer(|ev: On<Pointer<Press>>| {...})` 监听 Press 事件，验证 Observer 机制在当前项目中的可行性。

**实证发现**: `Pointer<Click>` 在 UI 按钮上不可靠（Press 和 Release 之间有微小移动时 Click 不生成）。`Pointer<Press>` 可靠触发。`bevy_ui_widgets::Button` 内部也使用 `Pointer<Press>` + `Pointer<Release>` 而非 `Pointer<Click>`。

**验证结论**: Observer 机制可行——可监听 Pointer 事件、可访问 ECS 资源、事件冒泡正常工作。验证代码（`observer.rs`）在 Phase 1a 完成后删除。

## Risks / Trade-offs

**[HoverMap 在 Update 阶段未更新]** → 概率极低。Picking 在 `PreUpdate` 阶段运行，`selection_click_system` 在 `Update` 阶段运行，时序正确。

**[HoverMap 包含非 UI 实体导致误判]** → 概率低。游戏实体无 `Pickable` 组件，UI 节点默认 `should_block_lower: true`，光标在 UI 上时游戏实体不会出现在 HoverMap 中。`is_cursor_over_ui` 通过检查 `Node` 组件进一步过滤。

**[Observer 闭包调试困难]** → 可接受。Phase 1a 仅用于验证，Observer 内业务逻辑抽离为独立函数，闭包只做事件转发。

**[bevy_ui_widgets 仍为 experimental]** → 不影响。Phase 1a 使用原生 Observer + `Pointer<Press>`，不引入 `bevy_ui_widgets`。Phase 2 是否引入取决于 Phase 1a 的实证结果。

**[删除 blocker 后 seek panel 行为变化]** → 无影响。seek panel 系统的 blocker 参数仅用于防止穿透，删除后 HoverMap 检查在 selection 层面统一处理，seek panel 功能不受影响。

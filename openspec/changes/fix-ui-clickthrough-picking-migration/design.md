## Context

当前项目使用 `Input<MouseButton>` + 手动 `UiFocusBlocker` 资源处理 UI 点击穿透。`UiFocusBlocker` 每帧重置为 `false`，各 UI 系统在检测到 `Interaction::Pressed` 时设置 `blocked = true`，`selection_click_system` 检查该标志后决定是否处理游戏世界点击。

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

**选择**: 在 `selection_click_system` 和 `command_issue_system` 中查询 `Res<HoverMap>`，判断光标是否在 UI 上。

**备选方案**:
- A) 为每个 UI 系统手动补 blocker → 维护成本高，容易遗漏
- B) 查询 `Query<&Interaction>` 判断是否有 UI 被 hover → 依赖 Interaction 组件，未来可能废弃
- C) 把 selection 系统也改为 Pointer 事件 → 需要游戏实体 pickable，改动大

**理由**: HoverMap 是 Picking 系统的公开 API，自动维护，无需手动设置。UI 节点默认 `should_block_lower: true`，光标在 UI 上时 HoverMap 中只有 UI 实体，不会包含游戏实体。这是最轻量、最符合引擎设计意图的方案。

### Decision 2: Observer 验证采用全局 Observer + Pointer<Click>

**选择**: 用 `app.add_observer(|ev: On<Pointer<Click>>| {...})` 监听所有点击事件，在 Observer 内通过 Query 判断是否命中目标按钮。

**备选方案**:
- A) 实体级 Observer `.observe(...)` → 需要在 spawn 时注册，改动大
- B) 自定义事件桥接 → 增加不必要的间接层
- C) 仅验证 Observer 机制，不涉及 Pointer → 验证不充分

**理由**: 全局 Observer 最简单，不需要修改 spawn 代码。`Pointer<Click>` 在 UI 按钮上会正确触发（已从源码确认），`ev.entity()` 返回被点击的按钮实体。Observer 可访问 `ResMut`、`Query` 等标准系统参数（已从源码确认）。

### Decision 3: Phase 1a 和 Phase 1b 作为独立验证步骤

**选择**: Phase 1a 验证 Observer 机制（menu 按钮），Phase 1b 验证穿透修复（HoverMap 替代 blocker）。

**理由**: 两个验证目标独立。Phase 1a 不涉及穿透修复，Phase 1b 不涉及 Observer。分开验证降低单步风险，失败时可独立回滚。Phase 1a 的 Observer 验证为 Phase 2 的 bevy_ui_widgets 迁移提供实证数据。

## Risks / Trade-offs

**[HoverMap 在 Update 阶段未更新]** → 概率极低。Picking 在 `PreUpdate` 阶段运行，`selection_click_system` 在 `Update` 阶段运行，时序正确。

**[HoverMap 包含非 UI 实体导致误判]** → 概率低。游戏实体无 `Pickable` 组件，UI 节点默认 `should_block_lower: true`，光标在 UI 上时游戏实体不会出现在 HoverMap 中。

**[Observer 闭包调试困难]** → 可接受。Phase 1a 仅用于验证，Observer 内业务逻辑抽离为独立函数，闭包只做事件转发。

**[bevy_ui_widgets 仍为 experimental]** → 不影响。Phase 1a 使用原生 Observer + `Pointer<Click>`，不引入 `bevy_ui_widgets`。Phase 2 是否引入取决于 Phase 1a 的实证结果。

**[删除 blocker 后 seek panel 行为变化]** → 无影响。seek panel 系统的 blocker 参数仅用于防止穿透，删除后 HoverMap 检查在 selection 层面统一处理，seek panel 功能不受影响。

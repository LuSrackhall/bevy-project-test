## Context

当前 UI 层使用 Bevy 旧版 `bevy_ui::Button` + `Interaction` 组件轮询模式。Phase 1 已完成穿透修复（`Interaction::Pressed` 替代 `UiFocusBlocker`）和 Observer 机制验证（`Pointer<Press>` 可靠）。代码库中存在 10 个按钮系统，全部使用 `Query<(&Marker, &Interaction), Changed<Interaction>>` 模式。

Bevy 0.18 存在三套并行的交互系统：
- `Interaction`（旧版，由 `ui_focus_system` 驱动）
- `PickingInteraction`（Picking 系统自动维护，与 Interaction 同构）
- `Pressed` + `Activate`（`bevy_ui_widgets` 的 Observer 模式）

## Goals / Non-Goals

**Goals:**

- 全量迁移到 `bevy_ui_widgets::Button` + `Activate` Observer 模式
- 用 `PickingInteraction` 统一过渡期的交互检测，消除双真相源
- 为所有按钮提供视觉反馈（hover/pressed 状态）
- 保持穿透保护（`is_any_ui_pressed`）在迁移期间不中断
- 符合 `ui/CLAUDE.md` 宪法：事件驱动、语义事件、行为与表现解耦

**Non-Goals:**

- 不修改仿真层（`simulation`）
- 不改变游戏逻辑
- 不实现完整文本编辑功能（输入框保留手写数字逻辑）
- 不处理拖拽框选的穿透（后续单独处理）

## Decisions

### Decision 1: 用 PickingInteraction 替代 Interaction 作为过渡方案

**选择**: 在 Phase 1.5 中，将所有 `Query<&Interaction>` 替换为 `Query<&PickingInteraction>`。

**备选方案**:
- A) 保持 Interaction 直到全量迁移 → 双真相源，bevy_ui_widgets::Button 不提供 Interaction
- B) 直接迁移到 Pressed → 混合期部分按钮没有 Pressed，穿透保护失效
- C) 用 Pointer<Over>/Out Observer → 与 is_any_ui_pressed 形成双真相源

**理由**: `PickingInteraction` 与 `Interaction` 几乎同构（Hovered/Pressed/None），API 迁移成本极低。它由 Picking 系统自动维护，无需手动管理。`bevy_ui_widgets::Button` 不提供 `Interaction`，但 Picking 系统仍提供 `PickingInteraction`。这是消除双真相源的最安全路径。

### Decision 2: Activate Observer 替代 Interaction 轮询

**选择**: 用 `bevy_ui_widgets::Button` + `Activate` Observer 替代 `Query<(&Marker, &Interaction), Changed<Interaction>>` 模式。

**备选方案**:
- A) 用 Pointer<Press> Observer → Phase 1a 已验证可行，但 Activate 是 bevy_ui_widgets 的语义事件，更符合架构
- B) 保持 Changed<Interaction> → 违反宪法"React to semantic UI events"
- C) 用 Changed<Pressed> → Pressed 是"按住中"，不是"点击完成"，语义不匹配

**理由**: `Activate` 是 `bevy_ui_widgets::Button` 的语义事件，表示"用户完成了一次点击操作"。它由 Button 内部的 Observer 链管理（Press → Release → Click → Activate），比手动处理 Pointer 事件更健壮。

### Decision 3: 集中式 button_style_system 处理视觉反馈

**选择**: 用 `ButtonTheme` 组件 + 集中式 `button_style_system` 处理所有按钮的视觉反馈。

**备选方案**:
- A) Observer 驱动样式 → 20+ 按钮 × 4 种状态 = 80 个 Observer，样式逻辑碎片化
- B) 每个按钮类型单独的样式系统 → 代码重复，维护成本高
- C) 不做视觉反馈 → 违反用户体验基本要求

**理由**: 样式是交互状态的纯函数投影，应集中管理。`ButtonTheme` 组件允许不同按钮有不同样式，系统逻辑统一。这符合"presentation 与 behavior 解耦"原则。

### Decision 4: is_any_ui_pressed 分阶段迁移

**选择**: 混合期同时检查 `PickingInteraction::Pressed` 和 `Pressed` 组件，全量迁移后简化为只检查 `Pressed`。

**理由**: `PickingInteraction` 在混合期覆盖未迁移的按钮，`Pressed` 覆盖已迁移的按钮。全量迁移后所有按钮都有 `Pressed`，可以删除 `PickingInteraction` 依赖。时序验证：`PreUpdate`（Picking 系统更新）先于 `Update`（selection 系统检查），无漏检窗口。

### Decision 5: Seek Panel 输入框保留手写逻辑

**选择**: 范围输入框保留手写数字捕获逻辑，仅将 `Interaction` 轮询替换为 Observer 化。

**备选方案**:
- A) 迁移到 `EditableText` → 需要数字过滤、最大长度限制，复杂度高
- B) 自定义 NumericInput widget → 投入产出比低

**理由**: 当前输入框只接受 0-9999 的数字，手写逻辑简单明确。`EditableText` 是通用文本编辑器，引入不必要的复杂度。Observer 化（用 `Pointer<Press>` 触发激活）足以消除轮询。

## Risks / Trade-offs

**[experimental feature 稳定性]** → `bevy_ui_widgets` 标记为 experimental，API 可能大幅变更。缓解：将 widget 用法封装在独立的 builder/spawner 函数中，便于未来 API 变更时集中修改。

**[PickingInteraction 与 bevy_ui_widgets 的兼容性]** → PickingInteraction 由 Picking 系统自动维护，与 bevy_ui_widgets 使用同一套 Picking 管线，兼容性风险低。

**[MenuPopup 定位策略]** → 当前下拉菜单通过 `PositionType::Absolute` + `bottom: Val::Px(28.0)` 定位在 trigger 上方。需确认 MenuPopup 的默认定位策略是否支持向上弹出。

**[混合期双系统共存]** → Phase 2a-2c 期间，部分按钮使用 bevy_ui_widgets::Button，部分使用旧版 Button。两者使用同一套 Picking 管线，不会互相干扰。

**[button_style_system 与 PickingInteraction 的时序]** → `button_style_system` 需要在 `ui_focus_system` 之后运行，以确保 `PickingInteraction` 已更新。通过系统排序约束解决。

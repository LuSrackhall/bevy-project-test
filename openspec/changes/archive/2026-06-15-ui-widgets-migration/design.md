## Context

当前 UI 层使用 Bevy 旧版 `bevy_ui::Button` + `Interaction` 组件轮询模式。Phase 1 已完成穿透修复（`Interaction::Pressed` 替代 `UiFocusBlocker`）和 Observer 机制验证（`Pointer<Press>` 可靠）。代码库中存在 10 个按钮系统，全部使用 `Query<(&Marker, &Interaction), Changed<Interaction>>` 模式。

## Goals / Non-Goals

**Goals:**

- 一次性全量迁移到 `bevy_ui_widgets::Button` + `Activate` Observer 模式
- 同步切换 `is_any_ui_pressed` 口径，消除混合期
- 为所有按钮提供视觉反馈（hover/pressed 状态）
- 符合 `ui/CLAUDE.md` 宪法：事件驱动、语义事件、行为与表现解耦

**Non-Goals:**

- 不修改仿真层（`simulation`）
- 不改变游戏逻辑
- 不实现完整文本编辑功能（输入框保留手写数字逻辑）
- 不处理拖拽框选的穿透（后续单独处理）

## Decisions

### Decision 1: 一次性迁移所有按钮（不分 Phase）

**选择**: 在单个 Phase 中迁移所有 25 个按钮，同步切换 `is_any_ui_pressed` 口径。

**备选方案**:
- A) 分步迁移（menu → HUD → Seek Panel）→ `is_any_ui_pressed` 全局守卫无法渐进式迁移，必然产生混合期
- B) PickingInteraction 过渡层 → 已验证失败（PickingInteraction 无 require，只在 hover 时才插入）
- C) 双检查方案（同时检查 Interaction 和 Pressed）→ 可行但增加无意义的过渡代码

**理由**: `is_any_ui_pressed` 是全局守卫，查询世界中所有 UI 实体的组件状态。只要世界中同时存在两种按钮组件体系，守卫就必须同时理解两者。一次性迁移消除这个矛盾。按钮迁移逻辑高度同质（都是 `Interaction` → `Activate` Observer），拆开不降低复杂度。

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

### Decision 4: 穿透保护使用 HoverMap

**选择**: 将 `is_any_ui_pressed` 替换为 `is_cursor_over_ui`，使用 `HoverMap` 查询光标是否在 UI 上。

**实际实现**: `is_cursor_over_ui` 遍历 `HoverMap` 中鼠标指针下的所有实体，检查是否有任何实体带有 `Node` 组件。

**备选方案**:
- A) `Query<&Pressed>` → `Pressed` 只在 `bevy_ui_widgets::Button` 上存在，`MenuButton`/`MenuItem` 不使用
- B) `Query<&PickingInteraction>` → 不自动插入到未被 hover 的实体，时序不可靠
- C) `Query<&Interaction>` → `bevy_ui_widgets::Button` 不提供 `Interaction` 组件

**理由**: `HoverMap` 由 Picking 系统自动维护，覆盖所有 UI 节点。透明容器添加 `Pickable::IGNORE` 后不阻挡下层，`HoverMap` 只包含实际的 UI 元素。这是最通用的方案，不依赖特定组件类型。

### Decision 5: Seek Panel 输入框保留手写逻辑

**选择**: 范围输入框保留手写数字捕获逻辑，仅将 `Interaction` 轮询替换为 Observer 化。

**备选方案**:
- A) 迁移到 `EditableText` → 需要数字过滤、最大长度限制，复杂度高
- B) 自定义 NumericInput widget → 投入产出比低

**理由**: 当前输入框只接受 0-9999 的数字，手写逻辑简单明确。`EditableText` 是通用文本编辑器，引入不必要的复杂度。Observer 化（用 `Pointer<Press>` 触发激活）足以消除轮询。

## Risks / Trade-offs

**[experimental feature 稳定性]** → `bevy_ui_widgets` 标记为 experimental，API 可能大幅变更。缓解：将 widget 用法封装在独立的 builder/spawner 函数中，便于未来 API 变更时集中修改。

**[全量迁移失败风险]** → Phase 2a 是单个大变更，如果失败需要 git revert。缓解：Phase 内按文件拆 commit，失败时 revert 到最近的 checkpoint。编译通过后必须运行验证再 merge。

**[MenuPopup 定位策略]** → 当前下拉菜单通过 `PositionType::Absolute` + `bottom: Val::Px(28.0)` 定位在 trigger 上方。需确认 MenuPopup 的默认定位策略是否支持向上弹出。

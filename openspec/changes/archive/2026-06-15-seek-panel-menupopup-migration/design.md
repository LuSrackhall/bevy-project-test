## Context

当前 Seek Panel 的下拉菜单使用手写实现：
- `Display::None`/`Display::Flex` 切换可见性
- `SeekPanelState.dropdown_open` 管理状态
- `Pointer<Click>` observer 处理选项点击（不工作）
- `seek_panel_dropdown_system` 处理关闭逻辑

Bevy 0.18 的 picking 时序问题导致 `Display::None` → `Display::Flex` 切换后，picking 系统在同一帧读到旧的 `size == Vec2::ZERO`，跳过该节点。这是根本性的架构问题，不是 bug。

`bevy_ui_widgets::MenuPopup` 通过 spawn/despawn 管理 popup 生命周期，新实体诞生时就有正确的布局，完全规避了此时序问题。

## Goals / Non-Goals

**Goals:**

- 用 `MenuPopup` + `MenuItem` 替代手写下拉菜单
- 注册必要的插件（MenuPlugin、PopoverPlugin、TabNavigationPlugin、InputDispatchPlugin）
- 删除 `seek_panel_dropdown_system` 和相关 workaround
- 简化 `SeekPanelState`

**Non-Goals:**

- 不迁移输入框（保留手写数字逻辑）
- 不迁移下发按钮（已使用 Activate Observer）
- 不修改其他 UI 系统

## Decisions

### Decision 1: 使用 MenuPopup 替代 Display 切换

**选择**: 用 `MenuPopup` + `MenuItem` 的 spawn/despawn 模式替代 `Display::None/Flex` 切换。

**备选方案**:
- A) 改用 `Visibility::Hidden` → 不解决 picking 问题（InheritedVisibility 同样被跳过）
- B) 手动 `contains_point` 检测 → 极复杂，重造 picking 后端
- C) 延迟一帧响应 → 用户体验差

**理由**: MenuPopup 是官方推荐的 popup 实现，通过实体生命周期管理规避了所有时序问题。

### Decision 2: 注册 InputDispatchPlugin 提供 InputFocus 资源

**选择**: 注册 `InputDispatchPlugin` + `TabNavigationPlugin`，为 `MenuPlugin` 提供 `InputFocus` 资源。

**理由**: `MenuPlugin` 的 `menu_acquire_focus` 系统直接以 `ResMut<InputFocus>` 访问资源。`InputFocus` 由 `InputDispatchPlugin` 初始化。这是硬性依赖。

### Decision 3: 保留 seek_panel_input_system 的键盘捕获

**选择**: 范围输入框的键盘捕获逻辑保留不变，但需要处理 InputFocus 与全局键盘输入的冲突。

**风险**: 菜单打开时，键盘事件可能被 InputFocus 路由到菜单项，导致范围输入不响应。缓解：菜单关闭时清除 InputFocus。

## Risks / Trade-offs

**[experimental feature 稳定性]** → 已在使用 Button，风险不增加。

**[InputFocus 与键盘输入冲突]** → 菜单打开时键盘事件被 InputFocus 拦截。缓解：菜单关闭时清除焦点。

**[向上弹出定位]** → Popover 支持 `PopoverSide::Top`。需实际测试确认。

**[实体层级变化]** → MenuPopup 需要 anchor → MenuButton + MenuPopup → MenuItem 的层级结构。与当前的 flat 结构不同。

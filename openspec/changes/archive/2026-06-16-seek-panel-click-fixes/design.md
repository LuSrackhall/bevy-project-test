## Context

两个独立的 UX bug。

### Bug 1: 光标不立即显示

`seek_panel_input_system` 中，点击输入框设置 `input_active = true`，但显示更新逻辑在同一个 if-else 分支的末尾。问题是：点击帧设置 `input_active = true`，但该帧的后续代码检查 `state.input_active` 时走的是"active"分支，会显示光标。实际上代码逻辑应该是正确的——需要确认是否是显示更新的时序问题。

实际上看代码：点击帧 `input_active` 被设为 true，然后后续 `if state.input_active` 分支检查键盘输入（该帧无键盘输入所以不更新），最后没有 else 分支来更新显示。需要在 `input_active` 刚变为 true 时立即更新显示。

### Bug 2: 点击穿透

`selection_click_system` 的 UI 守卫检查 `interaction_query.iter().any(|i| *i != Interaction::None)`。这检查的是**全局**是否有任何 UI 元素被交互，而不是检查**点击位置**是否在 UI 上。

问题：如果点击位置在 Text 节点（如模式标签"索敌"）上，该节点没有 `Interaction` 组件，守卫不会触发。点击穿透到游戏世界，没有找到单位，选区被清空。

修复方案：在 `PreUpdate` 阶段用一个 `Res` 标记"本帧是否有 UI 交互"，在 `Update` 阶段的选择系统中检查该标记。或者更简单：检查鼠标点击位置是否在任何带有 `Node` 组件的 UI 矩形内。

## ADDED Requirements

### Requirement: SeekScopeDropdown popup appears without position flicker
SeekScopeDropdown 的弹出面板 SHALL 在布局计算完成前不可见，popup 仅在 `ComputedNode.size` 非零后才变为可见状态。`reveal_popover` 系统 SHALL 运行在 `UiSystems::Layout` 之后，检查 `ComputedNode.size() != Vec2::ZERO` 后移除 `PopoverReady` 标记组件，使 popup 于下一帧由 Popover 系统正常显示。

#### Scenario: First open shows popup at correct position
- **WHEN** 玩家点击 SeekScopeDropdown 按钮触发下拉菜单展开
- **THEN** popup 面板首次出现时已在正确位置（按钮上方或下方），无任何位置跳变或闪烁

#### Scenario: PopoverReady marker lifecycle
- **WHEN** popup 实体被 spawn（observer 中 `commands.spawn(...)`）
- **THEN** popup 实体同时携带 `PopoverReady` 标记组件，直到 `ComputedNode.size` 非零后该标记被移除

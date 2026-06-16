## ADDED Requirements

### Requirement: SeekScopeDropdown popup appears without position flicker
SeekScopeDropdown 的弹出面板 SHALL 在展开时直接出现在正确位置，无位置跳变或闪烁。实现方式为在 observer 中手动计算 popup 的 `top` 位置（基于 anchor 的 `UiGlobalTransform` 和窗口尺寸），不使用 Bevy 的 Popover 系统。

#### Scenario: First open shows popup at correct position
- **WHEN** 玩家点击 SeekScopeDropdown 按钮触发下拉菜单展开
- **THEN** popup 面板首次出现时已在正确位置（按钮上方或下方），无任何位置跳变或闪烁

#### Scenario: Popup position adapts to available space
- **WHEN** 触发按钮靠近窗口底部，上方空间不足以容纳 popup
- **THEN** popup 出现在按钮下方；否则优先出现在按钮上方

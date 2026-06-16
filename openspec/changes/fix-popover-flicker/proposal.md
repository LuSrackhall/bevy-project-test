## Why

SeekScopeDropdown 的下拉面板在展开瞬间出现"从按钮下方快速上移"的闪烁。根因是 Bevy Popover 系统在 `UiSystems::Prepare` 阶段用 `ComputedNode.size = Vec2::ZERO`（布局尚未计算）定位 popup，下一帧尺寸真实后重新计算位置导致跳变。用户体验受损，需修复。

## What Changes

- 在 `hud.rs` 中添加 `PopoverReady` 标记组件，随 popup 实体 spawn 时一并添加
- 新增 `reveal_popover` 系统，运行在 `PostUpdate` 的 `UiSystems::Layout` 之后，当 `ComputedNode.size != Vec2::ZERO` 时移除标记，使 popup 首次可见
- 在 `mod.rs` 中注册该系统
- 不改变 Popover 配置、不改变其他 UI 面板行为

## Capabilities

### New Capabilities
（无全新能力，仅为现有 UI 系统的 bug 修复）

### Modified Capabilities
- `ui-system-fixes`: 新增一条需求——SeekScopeDropdown 弹出面板在展开时不得出现位置跳变闪烁，popup 须在布局计算完成后才可见

## Impact

- **受影响文件**：`crates/render_view/src/ui/hud.rs`、`crates/render_view/src/ui/mod.rs`
- **API 影响**：无公开 API 变更，仅内部实现
- **性能影响**：新增一个轻量级系统（遍历带 `PopoverReady` 的实体），几乎无开销
- **兼容性**：依赖 Bevy 公开 API（`ComputedNode.size()`、`UiSystems::Layout`），版本升级友好
- **风险**：popup 出现延迟一帧（~16ms@60fps），用户无法感知

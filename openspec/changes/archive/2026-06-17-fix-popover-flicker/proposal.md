## Why

SeekScopeDropdown 的下拉面板在展开瞬间出现"从按钮下方快速上移"的闪烁。根因是 Bevy 的 Popover 系统在 `UiSystems::Prepare` 阶段用零尺寸计算位置，下一帧布局完成后重新计算导致位置跳变。多次基于 Visibility/Display 的修复尝试均因 Bevy 内部系统时序冲突而失败，最终放弃 Popover 改为手动定位。

## What Changes

- 重写 `hud.rs` 中的 dropdown observer：通过 `ChildOf` 从触发按钮找到 anchor，查询 `UiGlobalTransform` 获取屏幕位置，手动计算 popup 的 `top` 位置
- 移除 `Popover` 组件及相关导入（`PopoverPlacement`、`PopoverSide`、`PopoverAlign`）
- 移除了中间方案的所有残留代码（`PopoverReady` 标记组件、`reveal_popover` 系统、`MenuTextMarker` 等）
- 不改变其他 UI 面板行为

## Capabilities

### New Capabilities
（无全新能力，仅为现有 UI 系统的 bug 修复）

### Modified Capabilities
- `ui-system-fixes`: SeekScopeDropdown 弹出面板在展开时直接出现在正确位置，无位置跳变或闪烁

## Impact

- **受影响文件**：`crates/render_view/src/ui/hud.rs`、`crates/render_view/src/ui/mod.rs`
- **API 影响**：无公开 API 变更，仅内部实现
- **性能影响**：移除了一个系统（reveal_popover），改为 observer 内直接计算，无额外开销
- **兼容性**：使用 Bevy 公开 API（`UiGlobalTransform`、`ComputedNode`、`ComputedUiRenderTargetInfo`），版本升级友好
- **风险**：丢失 Popover 的自动边缘裁剪，改为简单的上方/下方选择逻辑

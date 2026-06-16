## Context

`hud.rs` 中的 SeekScopeDropdown 原本使用 Bevy 的 `MenuButton` + `MenuPopup` + `Popover` 系统实现下拉菜单定位。展开时 popup 出现位置跳变闪烁（从按钮下方快速跳到上方）。

根因是 Bevy Popover 系统在 `UiSystems::Prepare` 阶段设置 `Visibility::Visible` 并计算位置，但此时 `ComputedNode.size` 尚未计算（为 `Vec2::ZERO`），导致位置基于零尺寸计算。下一帧布局完成后位置跳变。

经过 5 轮修复尝试，发现 Bevy 的 visibility/display 控制存在无法绕过的系统时序冲突：
- `Visibility::Hidden` 被 Popover 覆盖为 `Visible`
- `Display::None` 导致 Popover 无法处理实体
- `alpha=0` 仍有半透明渲染伪影
- `CheckVisibility` 在 `PostLayout` 之前运行，无法在渲染前拦截

最终方案：放弃 Popover 系统，改为手动计算 popup 位置。

## Goals / Non-Goals

**Goals:**
- 消除 SeekScopeDropdown 展开时的位置跳变闪烁
- 保持下拉菜单的展开/收起功能正常
- 保持合适的菜单宽度

**Non-Goals:**
- 不使用 Bevy 的 Popover 系统（已证明无法可靠控制显示时机）
- 不添加动画或过渡效果

## Decisions

### 决策 1：放弃 Popover，改为手动计算位置

**选择**：在 observer 中通过 `ChildOf` 找到 anchor 父节点，查询 `UiGlobalTransform` 获取屏幕位置，根据窗口高度手动计算 popup 的 `top` 位置。

**理由**：
- Bevy 的 Popover 系统与 UI 渲染管线存在无法绕过的时序冲突
- 所有基于 Visibility/Display/alpha 的延迟显示方案均失败
- 手动定位完全绕过这些时序问题

**替代方案**（均已失败）：
- 方案 A：`PopoverReady` 标记 + `reveal_popover` 系统 + `Visibility::Hidden` → 被 Popover 覆盖
- 方案 B：`Display::None` + 标记 → Popover 无法处理 Display::None 的实体
- 方案 C：alpha=0 透明颜色 → 仍有半透明渲染伪影
- 方案 D：两步标记（PopoverPositioned + PopoverReady）→ 仍有时序问题

### 决策 2：popup 添加到触发按钮子节点

**选择**：`commands.entity(ev.source).add_child(popup_entity)`。

**理由**：关闭逻辑通过 `q_trigger.get(ev.source)` 查找触发按钮的子节点定位已有 popup。

### 决策 3：使用估算高度计算位置

**选择**：`est_popup_h = 5.0 * 24.0`（5 个选项 × 每项约 24px）。

**理由**：popup 尚未布局时无法获取真实高度，使用估算值。选项数量固定，估算足够准确。

## Risks / Trade-offs

**[估算高度不精确]** 如果菜单项数量或字体大小变化，估算高度可能不准。
→ 缓解：当前选项数量固定（5 个），估算值合理。

**[丢失 Popover 的边缘裁剪]** Popover 会自动避免 popup 超出窗口边缘。
→ 缓解：通过上方/下方选择逻辑已处理主要场景。

**[丢失 Popover 的点击外部关闭]** 原本依赖 Popover + MenuPopup 的自动关闭机制。
→ 缓解：`menu_on_lose_focus` 系统仍然处理焦点丢失时的关闭。

## 实施方案

### 修改文件
- `crates/render_view/src/ui/hud.rs`：重写 observer 为手动定位，移除 Popover 相关代码
- `crates/render_view/src/ui/mod.rs`：移除 reveal_popover 注册和 UiSystems 导入

### 具体修改

**1. observer 重写（`hud.rs`）：**
- 查询参数：`q_trigger: Query<(&ChildOf, &Children)>`、`q_anchor: Query<(&UiGlobalTransform, &ComputedNode)>`、`q_popup`、`q_window`
- 通过 `ChildOf` 从 `ev.source`（触发按钮）找到 anchor 父节点
- 查询 anchor 的 `UiGlobalTransform.affine().translation.y` 和 `ComputedNode.size()`
- 查询 `ComputedUiRenderTargetInfo` 获取窗口高度
- 根据空间计算 `top` 值（上方优先）

**2. 移除的代码：**
- `PopoverReady`、`MenuTextMarker` 标记组件
- `reveal_popover` 系统及其注册
- Popover 相关导入（`PopoverPlacement`、`PopoverSide`、`PopoverAlign`）

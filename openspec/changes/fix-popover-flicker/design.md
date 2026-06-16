## Context

`hud.rs` 中的 SeekScopeDropdown 原本使用 Bevy 的 `MenuButton` + `MenuPopup` + `Popover` 系统实现下拉菜单定位。展开时 popup 出现位置跳变闪烁。

经过深入调查，根因是 Bevy 的 Popover 系统在 `UiSystems::Prepare` 阶段设置 `Visibility::Visible` 和计算位置，但此时布局尚未完成（`ComputedNode.size = Vec2::ZERO`），导致位置计算错误。后续帧中布局完成后位置跳变。

多次修复尝试（`Visibility::Hidden`、`Display::None`、alpha=0 透明颜色）均因 Bevy 内部系统时序问题（`CheckVisibility` 在 `PostLayout` 之前运行、Popover 覆盖 Visibility 等）而失败。

## Goals / Non-Goals

**Goals:**
- 消除 SeekScopeDropdown 展开时的位置跳变闪烁
- 保持下拉菜单的展开/收起功能正常
- 保持合适的菜单宽度

**Non-Goals:**
- 不使用 Bevy 的 Popover 系统（已证明无法可靠控制显示时机）

## Decisions

### 决策 1：放弃 Popover，改为手动计算位置

**选择**：在 observer 中直接计算 popup 的 `top` 位置，不使用 `Popover` 组件。

**理由**：
- Popover 系统的 visibility 管理与 UI 渲染管线存在时序冲突
- 所有基于 Visibility/Display 的延迟显示方案均因 `CheckVisibility` 的执行顺序而失败
- 手动定位完全绕过这些时序问题，popup 直接出现在正确位置

**实现**：
- 通过 `ChildOf` 从触发按钮找到 anchor 父节点
- 查询 anchor 的 `UiGlobalTransform.affine().translation.y` 获取触发位置
- 查询 `ComputedUiRenderTargetInfo` 获取窗口高度
- 根据空间决定弹出方向：上方空间足够则放上方，否则放下方

### 决策 2：popup 添加到触发按钮子节点

**选择**：`commands.entity(ev.source).add_child(popup_entity)` 而非 anchor。

**理由**：关闭逻辑通过 `q_anchor.get(ev.source)` 查找触发按钮的子节点来定位已有 popup。如果 popup 在 anchor 子节点中，关闭逻辑找不到。

### 决策 3：使用 estimated popup height 计算位置

**选择**：`est_popup_h = 5.0 * 24.0`（5 个选项 × 每项约 24px）。

**理由**：popup 尚未布局时无法获取真实高度，使用估算值。对于固定选项数量的菜单，估算足够准确。

## Risks / Trade-offs

**[估算高度不精确]** 如果菜单项数量或字体大小变化，估算高度可能不准。
→ 缓解：当前选项数量固定（5 个），估算值合理。

**[丢失 Popover 的边缘裁剪]** Popover 会自动避免 popup 超出窗口边缘，手动定位不处理此情况。
→ 缓解：通过上方/下方选择逻辑已处理主要场景。极端情况下（小窗口）可能溢出。

**[丢失 Popover 的点击外部关闭]** 原本依赖 Popover + MenuPopup 的自动关闭机制。
→ 缓解：`menu_on_lose_focus` 系统仍然处理焦点丢失时的关闭。

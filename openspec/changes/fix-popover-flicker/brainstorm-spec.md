## Context

`hud.rs` 中的 SeekScopeDropdown 使用 Bevy 的 `MenuButton` + `MenuPopup` + `Popover` 系统实现下拉菜单。按下按钮时动态 spawn 一个弹出面板（popup entity）。

Bevy Popover 系统（`bevy_ui_widgets::popover::position_popover`）运行在 `UiSystems::Prepare`（`PostUpdate` 阶段），它直接调用 `visibility.set_if_neq(Visibility::Visible)` 覆盖了 popup 的 `Visibility::Hidden`。此时 `ComputedNode.size` 尚未计算（为 `Vec2::ZERO`），Popover 用零尺寸计算位置——选择下方（Bottom）。下一帧布局计算完成后尺寸真实，Popover 重新计算位置可能选择上方（Top）→ 位置跳变 = 闪烁。

系统执行顺序：
```
PostUpdate:
  Prepare   ← Popover 运行（覆盖 visibility，用旧尺寸计算位置）
  Propagate
  Content
  Layout    ← ComputedNode 在此被赋予真实尺寸
  PostLayout ← 延迟显示系统运行点（尺寸已知，可决定是否显示）
```

## Goals / Non-Goals

**Goals:**
- 阻止 SeekScopeDropdown 的 popup 在尺寸未知时显示
- 只在 Popover 用真实尺寸正确定位后才让 popup 可见
- 消除"从按钮下方快速上移"的视觉闪烁

**Non-Goals:**
- 不修改 Bevy 的 Popover 系统本身
- 不改变其他下拉菜单或面板的行为
- 不添加动画或过渡效果

## Decisions

### 决策 1：使用 `PopoverReady` 标记组件 + 延迟显示系统

**选择**：添加 `PopoverReady` 标记组件到 popup 实体，新增 `reveal_popover` 系统在 `PostLayout` 阶段检查条件后移除标记。

**理由**：
- Bevy Popover 在 `Prepare` 阶段覆盖 `Visibility::Hidden` 为 `Visibility::Visible`，无法通过简单的 visibility 设置阻止显示
- 需要在 Popover 完成定位后、渲染前的窗口期介入
- `PostLayout` 阶段 `ComputedNode.size` 已有真实值，可以可靠地判断布局是否完成

**替代方案**：
- 方案 B（禁用 Popover 手动定位）：需自己处理窗口边缘、多显示器边界，且需重新实现点击外部关闭等行为
- 方案 C（预计算尺寸）：依赖字体度量精度，实现复杂度高，不能保证所有场景无闪烁

### 决策 2：使用 `ComputedNode.size != Vec2::ZERO` 作为布局完成判断

**选择**：检查 `ComputedNode.size()` 是否非零来判断布局是否已计算。

**理由**：
- `ComputedNode.size` 默认为 `Vec2::ZERO`，在 `UiSystems::Layout` 阶段由 `ui_layout_system` 赋予真实尺寸
- popup 包含文本子节点，布局计算后尺寸必然非零
- 使用公开 API，不依赖 Bevy 内部实现细节

### 决策 3：使用 `Visibility::Inherited` 状态过滤

**选择**：`reveal_popover` 系统只处理 `Visibility::Inherited` 的实体，跳过仍在 `Visibility::Visible`（Popover 设置）或 `Visibility::Hidden`（初始状态）的实体。

**理由**：
- Popover 在 `Prepare` 阶段设置 `Visibility::Visible`
- 如果在 `PostLayout` 将其改回 `Inherited`，下一帧 Popover 发现已是 `Visible` 不再设置，但我们的系统需要等 Popover 自然将 visibility 改为 `Inherited`（变化检测触发时）
- 实际上，`set_if_neq` 只在值不同时触发变化，所以 Popover 每帧都会设置 `Visible` 直到我们的系统介入

**修正说明**：实际上 `reveal_popover` 系统只需检查 `ComputedNode.size != Vec2::ZERO`，当条件满足时移除标记。此时 Popover 已用真实尺寸正确定位，popup 可安全显示。visibility 状态过滤是额外的安全保障。

## Risks / Trade-offs

**[一帧延迟]** popup 在按下按钮后 ~16ms（@60fps）才显示。
→ 缓解：比闪烁好得多，用户无法感知这一帧延迟。

**[Popover API 兼容性]** 如果 Bevy 未来版本改变 Popover 的 visibility 管理方式，可能需要调整。
→ 缓解：我们只使用公开 API（`ComputedNode.size()`、`Visibility`、`UiSystems::Layout`），不依赖内部实现。

**[快速点击边界情况]** 用户快速打开/关闭下拉菜单时，`PopoverReady` 组件随 entity 生命周期自动清理，不影响正确性。
→ 无需额外处理。

## 实施方案

### 修改文件
- `crates/render_view/src/ui/hud.rs`：添加组件 + 修改 spawn + 新增系统
- `crates/render_view/src/ui/mod.rs`：注册系统

### 具体修改

**1. `hud.rs` 标记组件区域（~line 150）添加：**
```rust
#[derive(Component)]
struct PopoverReady;
```

**2. `hud.rs` observer 的 popup spawn tuple（~line 362）添加 `PopoverReady`：**
```rust
let popup_entity = commands.spawn((
    Node { ... },
    MenuPopup::default(),
    Visibility::Hidden,
    PopoverReady,  // 新增
    BackgroundColor(...),
    GlobalZIndex(100),
    Popover { ... },
    OverrideClip,
))
```

**3. `hud.rs` 末尾新增系统：**
```rust
fn reveal_popover(
    mut commands: Commands,
    q: Query<(Entity, &ComputedNode), With<PopoverReady>>,
) {
    for (entity, node) in &q {
        if node.size() != Vec2::ZERO {
            commands.entity(entity).remove::<PopoverReady>();
        }
    }
}
```

**4. `mod.rs` 注册系统：**
```rust
.add_systems(PostUpdate, hud::reveal_popover.after(UiSystems::Layout))
```

需要在 `mod.rs` 中添加导入：
```rust
use bevy::ui::UiSystems;
```

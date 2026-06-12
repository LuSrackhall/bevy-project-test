## Context

`render_view/src/unit_info_bar.rs` 中的信息条系统使用 `bevy_prototype_lyon` 的 `ShapeBundle` 绘制血量条和经验条。背景矩形正常显示，但填充矩形（通过 `commands` despawn/respawn 创建）始终不可见。经排查，`ShapeBuilder::fill()` 对一次性创建的背景条有效，但对动态重建的填充条无效——可能是 `bevy_prototype_lyon` 的 mesh 生成或 parent 绑定存在 deferred commands 兼容性问题。

## Goals / Non-Goals

**Goals:**
- 将 HP fill / EXP fill 从 `ShapeBundle` 改为 Bevy 原生 `Sprite`
- 更新逻辑从 despawn+respawn 改为直接修改 `Sprite.custom_size`
- 确保填充条按实际占比正确显示

**Non-Goals:**
- 不更换背景条的渲染方式（`ShapeBundle` 已正常工作）
- 不修改信息条的组件架构、布局常量或颜色
- 不引入新依赖

## Decisions

### 1. 使用 `Sprite` 替代 `ShapeBundle` 绘制填充条

**选择：** HP fill 和 EXP fill 改用 `Sprite { color, custom_size, anchor }` 组件。

**原理：**
- `Sprite` 是 Bevy 最基础的渲染组件，行为完全可预测
- 更新时只需修改 `custom_size` 字段，无需 despawn/respawn
- `Sprite` 使用 `Anchor::Center` 原点，配合 Transform 偏移可实现左对齐
- 消除了 `bevy_prototype_lyon` 的 deferred commands 兼容性问题

**替代方案：**
- 继续调试 `ShapeBundle`：问题在库内部，不可控
- 改用 Bevy UI `Node`：需要屏幕空间投影，架构变更过大

### 2. 移除 `HpFill` / `ExpFill` marker 组件

**选择：** 填充条不再需要专用 marker 组件，`BarParts` 中的 entity ID 已足够定位。

**原理：** `Sprite` 实体直接通过 `BarParts.hp_fill` / `BarParts.exp_fill` 的 Entity ID 访问，无需 marker 过滤查询。

### 3. 背景条保持 `ShapeBundle`

**选择：** 背景条（HP bg / EXP bg）继续使用 `ShapeBundle`。

**原理：** 背景条已正常工作，无需改动。保持最小变更范围。

## Risks / Trade-offs

- **风险：** `Sprite` 的 `Anchor` 偏移计算与 `ShapeBundle` 的 `RectangleOrigin` 不一致 → 缓解：使用 `Anchor::Center` + Transform x 偏移，与之前的设计方案数学一致
- **权衡：** 填充条和背景条使用不同渲染技术 → 可接受：两者视觉效果一致（纯色矩形），用户无感知差异

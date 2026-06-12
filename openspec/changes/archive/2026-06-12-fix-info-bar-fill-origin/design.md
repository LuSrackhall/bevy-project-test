## Context

`render_view/src/unit_info_bar.rs` 中的信息条系统使用 `bevy_prototype_lyon` 的 `ShapeBundle` 绘制血量条和经验条。背景矩形使用 `RectangleOrigin::Center` 正常显示，但填充矩形使用 `RectangleOrigin::CustomCenter(Vec2)` 时，填充完全不可见。

经分析，`bevy_prototype_lyon 0.16` 的 `CustomCenter` 变体可能未正确将偏移量应用到生成的 `Path` 上，导致填充矩形被绘制在错误位置。

## Goals / Non-Goals

**Goals:**
- 修复血量条和经验条的填充显示，使其按实际占比正确渲染
- 保持现有信息条的布局、尺寸和颜色不变

**Non-Goals:**
- 不更换图形库（继续使用 `bevy_prototype_lyon`）
- 不修改信息条的组件架构或系统参数
- 不调整血条/经验条的像素尺寸或颜色

## Decisions

### 1. 将 `CustomCenter` 替换为 `Center` + Transform 偏移

**选择：** 改用 `RectangleOrigin::Center`，通过 `Transform.translation.x` 控制左对齐位置。

**原理：** `Center` 是 `bevy_prototype_lyon` 最基础、最广泛使用的原点模式，行为可靠。填充条的中心位置计算为 `-BAR_W/2 + fill_w/2`，左边缘自然对齐背景条左边缘。

**替代方案：**
- 改用 Bevy 原生 `Sprite`：改动过大，偏离设计文档的 ShapeBundle 选择
- 调试 `CustomCenter`：依赖库内部实现，不可控

**数学验证：**
- 背景条：`Center` 原点，Transform x=0 → 左边缘在 -20，右边缘在 +20
- 填充条（50% 宽度）：`Center` 原点，Transform x = -20 + 10 = -10 → 左边缘在 -20，右边缘在 0 ✓

### 2. 修改范围：仅改 4 处

4 处需要修改：
1. `create_bar` 中 HP fill 的 `origin` + `Transform`
2. `create_bar` 中 EXP fill 的 `origin` + `Transform`
3. `update_bar` 中 HP fill 的 `origin` + `Transform`
4. `update_bar` 中 EXP fill 的 `origin` + `Transform`

## Risks / Trade-offs

- **风险：** Transform 偏移后，填充条可能与背景条有 1px 间隙 → 缓解：使用相同的 y 坐标和 z 偏移（0.01），确保层叠正确
- **权衡：** 放弃 `CustomCenter` 的语义表达力 → 可接受：`Center` + Transform 偏移是更通用且可靠的做法

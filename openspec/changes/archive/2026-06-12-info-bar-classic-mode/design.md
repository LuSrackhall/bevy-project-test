## Context

`render_view/src/unit_info_bar.rs` 的信息条系统有三种显示模式（Always/Selected/Smart），通过 `InfoBarMode` 枚举和 `should_show` 逻辑控制。悬停检测在 `selection.rs` 中已有实现模式（`screen_to_world` + 距离比较）。

## Goals / Non-Goals

**Goals:**
- 新增 Classic 模式：仅选中或悬停的单位显示血条
- 修改 Selected 模式：增加悬停显示
- 悬停检测适用于所有有血条的单位（士兵+城池）

**Non-Goals:**
- 不创建设置面板 UI（仅 Ctrl+H 切换）
- 不修改 Smart/Always 模式
- 不引入新依赖

## Decisions

### 1. 悬停检测实现

**选择：** 在 `unit_info_bar_system` 中添加 `Query<&Window>` 和 `Query<(&Camera, &GlobalTransform), With<MainCamera>>`，将光标屏幕坐标转换为世界坐标，与所有单位位置比较距离。

**原理：** 复用 `selection.rs` 中已有的 `screen_to_world` 模式。悬停阈值使用与选中相同的距离（士兵 12px，城池使用 `CityRadius`）。

**替代方案：** 使用 Bevy 的 `Pointer` 事件系统 → 需要额外组件和事件监听，改动过大。

### 2. Classic 模式逻辑

**选择：** `should_show = is_selected || is_hovered`，不考虑 `hp_cur < hp_max` 或 `exp > 0`。

**原理：** Classic 模式的语义是"仅交互时显示"，与 Smart 模式的"状态变化时显示"完全不同。

### 3. Selected 模式增加悬停

**选择：** 将 Selected 模式的逻辑从 `is_selected` 改为 `is_selected || is_hovered`。

**原理：** 悬停是低侵入性的信息获取方式，Selected 模式增加悬停不会改变原有选中逻辑，只增加了一个额外的显示触发条件。

### 4. 模式循环顺序

**选择：** Always → Selected → Smart → Classic → Always

**原理：** Classic 作为新的"最精简"模式放在最后，从 Smart 切换到 Classic 符合"信息量递减"的直觉。

## Risks / Trade-offs

- **风险：** 每帧遍历所有单位比较距离，单位数量大时有性能开销 → 缓解：与选中系统相同的开销，已有先例
- **权衡：** Selected 模式增加悬停可能改变用户习惯 → 可接受：悬停是被动操作，不干扰原有选中逻辑

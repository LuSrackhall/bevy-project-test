## Why

单位头顶信息条的血量条和经验条填充矩形始终显示为空（血条全红、经验条全蓝），绿色/紫色填充完全不可见。根因是 `bevy_prototype_lyon 0.16` 的 `RectangleOrigin::CustomCenter` 变体未按预期偏移矩形中心，导致填充条被绘制在错误位置（可能覆盖在背景条下方或偏移到不可见区域）。

此 bug 使得信息条的核心功能（显示血量/经验值占比）完全失效，必须修复。

## What Changes

- 将信息条中 4 处 `RectangleOrigin::CustomCenter` 替换为 `RectangleOrigin::Center`
- 对应调整 4 处 `Transform` 的 x 坐标，通过位置偏移实现填充条左对齐
- 不涉及组件结构、系统参数、数据流或架构变更

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

无修改需求规格（这是纯实现层 bug 修复，不改变任何功能需求）。

## Impact

- 影响 crate: `render_view/`（仅 `unit_info_bar.rs` 一个文件）
- 影响范围: 4 处 `ShapeBuilder` 矩形原点 + 4 处 `Transform` 坐标
- 无新依赖引入
- 无 breaking change
- 不影响 `simulation`、`bevy_adapter`、`presentation` 层

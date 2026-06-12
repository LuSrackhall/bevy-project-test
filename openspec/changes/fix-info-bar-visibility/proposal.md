## Why

信息条的可见性控制存在两个相关 bug：

1. **城池血条不显示：** 城池在游戏开始时以 Smart 模式创建（满血满经验 → `should_show = false`），`create_bar` 将根实体和所有子实体都设为 `Visibility::Hidden`。之后即使切换到 Always 模式，子实体仍保持 `Hidden` 状态。

2. **切换模式后旧单位血条不显示：** 同一机制——在 `should_show = false` 时创建的 bar，切换模式后 `update_bar` 只修改根实体的 `Visibility`，子实体（填充条、文字）仍然 `Hidden`。

根因：`create_bar` 中 `Visibility::Hidden` 被同时应用于根实体和子实体，但 `update_bar` 只更新根实体。子实体应始终使用 `Visibility::Inherited` 以继承父级可见性。

## What Changes

- 修改 `create_bar`：子实体（文字、背景条、填充条）始终使用 `Visibility::Inherited`，只有根实体根据 `visible` 参数设置 `Hidden`/`Inherited`
- 不涉及组件结构、系统参数或数据流变更

## Capabilities

### New Capabilities

无。

### Modified Capabilities

无（纯实现层 bug 修复）。

## Impact

- 影响 crate: `render_view/`（仅 `unit_info_bar.rs` 的 `create_bar` 函数）
- 不影响其他层

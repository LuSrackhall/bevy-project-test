## Why

索敌面板存在 6 个相互关联的 bug，导致下拉框选项无法选中、范围输入框无法编辑。根因涉及 Bevy 拾取管线理解错误、实体 ID 存储错误、UI 点击穿透到游戏选择系统、以及系统执行顺序竞态。

## What Changes

- **修改** 弹出层从 `left: -9999` 改回 `Display::None`/`Display::Flex`（Bevy 拾取管线原生支持）
- **修改** `HudTexts` 中 `seek_scope_text` 和 `seek_range_text` 改为存储 Text 子实体 ID（而非 Button 父实体 ID）
- **修改** `selection_click_system` 增加 UI 交互检测，当光标在 UI 元素上时跳过游戏世界选择逻辑
- **修改** `seek_panel_mode_system` 在编辑态下不重置编辑状态
- **修改** 系统注册增加执行顺序约束，确保 HUD 系统在选择系统之前运行
- **修改** 下拉框 click-outside 逻辑增加安全守卫

## Capabilities

### New Capabilities

- `seek-panel-fix`: 索敌面板 6 个根因 bug 的综合修复

### Modified Capabilities

（无）

## Impact

- **render_view/src/ui/hud.rs**: 弹出层样式、文本实体存储、模式切换系统
- **render_view/src/selection.rs**: 增加 UI 交互守卫
- **render_view/src/ui/mod.rs**: 系统排序
- **render_view/src/lib.rs**: 系统排序

## Why

Bevy 0.18 原生 UI 的 `Interaction` 组件在动态显示/隐藏场景下不可靠：`Display::None` → `Display::Flex` 切换后子节点不会立即生成正确的 `Interaction` 状态，`Changed<Interaction>` 也无法正确触发。导致索敌面板的下拉选择框点击选项后无响应，以及范围输入框无法进入编辑模式。需要从根本上规避 Bevy 交互系统的缺陷，确保 UI 交互稳定可靠。

## What Changes

- **修改** 下拉弹出层的可见性控制：从 `Display::None`/`Display::Flex` 切换改为始终渲染 + 屏幕外偏移（`left: -9999`）隐藏，确保子节点 `Interaction` 始终正常工作
- **修改** 范围输入框的点击检测：从 `Changed<Interaction>` 过滤改为每帧检查 `Interaction::Pressed` + `mouse.just_pressed` 防抖
- **修改** 下发按钮的点击检测：同样移除 `Changed<Interaction>` 依赖
- **修改** 下拉触发器的点击检测：同样移除 `Changed<Interaction>` 依赖
- **新增** 输入防重入标志 `input_was_pressed`，防止持续按住鼠标时重复触发编辑模式
- **保留** 现有功能逻辑不变，仅修改交互检测层

## Capabilities

### New Capabilities

- `ui-interaction-reliability`: Bevy 原生 UI 交互可靠化方案——用位置偏移代替 Display 切换、用每帧状态检查代替 Changed 事件过滤

### Modified Capabilities

（无已有 spec 级别的行为变更）

## Impact

- **render_view/src/ui/hud.rs**: 修改 `seek_panel_dropdown_system`、`seek_panel_input_system`、`seek_panel_issue_system` 三个系统，以及 `setup_hud` 中弹出层的初始样式

## Why

之前在实现头顶信息条时，我们将底部面板（HUD）的血量、经验、等级显示移除了（任务 5.1-5.2）。但底部面板的信息在任意缩放比下都可读，是玩家查看单位详情的重要途径。头顶信息条和底部面板应并存——头顶提供快速战场感知，底部面板提供精确数值查看。

## What Changes

- 恢复士兵面板的 HP 文本、HP 填充条、EXP 文本、EXP 填充条、等级显示
- 恢复城池面板的等级、血量、经验显示
- 移除 "(HP/经验见头顶)" 提示文字
- 恢复查询中对 `Health` 和 `Level` 组件的读取

## Capabilities

### New Capabilities

无。

### Modified Capabilities

无（纯回退修改）。

## Impact

- 影响 crate: `render_view/`（仅 `ui/hud.rs`）
- 不影响其他层

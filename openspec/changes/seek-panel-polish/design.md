## Context

索敌面板基本功能已实现，用户反馈 4 个改进点。

## Decisions

### D1: 补充民兵选项

当前下拉选项为 `全体/步兵/弓兵/骑兵`，缺少 `民兵`。补充为 `全体/民兵/步兵/弓兵/骑兵` 五项。对应 `SeekScope::ByType(SoldierType::Militia)`。

### D2: 输入框实时生效

移除 `SeekPanelState.editing` 和 `input_buffer` 概念。输入框点击后直接进入"键盘输入捕获"状态，数字键直接追加到 `range_value` 的字符串表示。Backspace 删除末位。显示值始终等于 `range_value`。

简化后状态机：`SeekPanelState` 移除 `editing` 和 `input_buffer` 字段，新增 `input_active: bool`。输入框显示 `range_value`，输入时实时修改。

### D3: 模式标签

在 scope 下拉选择框左侧新增一个 Text 节点：
- 全局模式（无选中单位）：显示"索敌"
- 选择模式（有选中单位）：显示"选中"

存储在 `HudTexts.mode_label` 中，由 `seek_panel_mode_system` 更新。

### D4: 下拉选项实时数量

每个 `SeekScopeOption` 旁显示对应兵种数量。需要查询 simulation 世界：
- 全局模式：统计所有 Player 士兵按兵种分组
- 选择模式：统计选中的 UnitId 按兵种分组

数量文本存储在每个选项的 Text 子实体中，由 `seek_panel_dropdown_system` 每帧更新（仅在 dropdown_open 时）。

## ADDED Requirements

### Requirement: 输入框光标即时显示

点击输入框进入输入态时，SHALL 立即显示光标（`▌`），无需等待键盘输入。

#### Scenario: 点击后立即显示光标

- **WHEN** 用户点击输入框
- **THEN** 输入框文字立即变为"当前值▌"格式

### Requirement: 点击不穿透 UI

操作索敌面板（包括模式标签、空白区域）时，SHALL NOT 导致游戏世界选区变化。

#### Scenario: 点击模式标签不清空选区

- **WHEN** 选中了 3 个单位，用户点击"索敌"标签
- **THEN** 选区保持不变

#### Scenario: 点击输入框不清空选区

- **WHEN** 选中了 5 个单位，用户点击范围输入框
- **THEN** 选区保持不变

## ADDED Requirements

### Requirement: Toast 消息资源与显示

系统 SHALL 维护 `ToastMessage` 资源，包含 `text: String` 和 `remaining_ticks: u32`。顶部状态栏右侧 SHALL 显示当前 Toast 消息文字。消息显示持续 100 ticks（5 秒 @20Hz），期间每帧递减 `remaining_ticks`。归零时清空消息文字。新消息 SHALL 覆盖旧消息。

#### Scenario: 消息自动消失

- **WHEN** Toast 消息 `remaining_ticks` 从 1 递减到 0
- **THEN** 消息文字清空，不再显示

#### Scenario: 新消息覆盖旧消息

- **WHEN** 已有一条 Toast 消息（剩余 50 ticks），触发新消息
- **THEN** 新消息替换旧消息，`remaining_ticks` 重置为 100

### Requirement: 选中单位摘要提示

当 `SelectionState.selected_unit_ids` 变化且非空时，系统 SHALL 在顶部显示选中摘要。单一兵种时格式为"选中 N 个兵种名"。混合兵种时格式为"选中 N 个单位: 兵种1数量1 兵种2数量2"。

#### Scenario: 选中单一兵种

- **WHEN** 选中 3 个骑兵
- **THEN** Toast 消息为"选中 3 个骑兵"

#### Scenario: 选中混合兵种

- **WHEN** 选中 5 个单位：2 步兵 + 3 弓兵
- **THEN** Toast 消息为"选中 5 个单位: 步兵2 弓兵3"

#### Scenario: 选中单位为空不清除已有消息

- **WHEN** 清空选中（selected_unit_ids 变为空）
- **THEN** 不生成新 Toast 消息，已有消息保持直到自然过期

### Requirement: 命令下发确认提示

当索敌命令下发后，系统 SHALL 在顶部显示确认消息。格式根据模式和 scope 不同：

- 全局全体：`已下发全体索敌 范围{N}`
- 全局按兵种：`已下发{兵种名}索敌 范围{N}`
- 选中全体：`已下发选中全体({总人数})索敌 范围{N}`
- 选中按兵种：`已下发选中{兵种名}({人数})索敌 范围{N}`

#### Scenario: 全局全体索敌确认

- **WHEN** 全局模式 scope=All range=30 下发命令
- **THEN** Toast 消息为"已下发全体索敌 范围30"

#### Scenario: 全局按兵种索敌确认

- **WHEN** 全局模式 scope=ByType(Infantry) range=20 下发命令
- **THEN** Toast 消息为"已下发步兵索敌 范围20"

#### Scenario: 选中全体索敌确认

- **WHEN** 选择模式，选中 5 个单位，scope=All range=30 下发命令
- **THEN** Toast 消息为"已下发选中全体(5)索敌 范围30"

#### Scenario: 选中按兵种索敌确认

- **WHEN** 选择模式，选中 3 个骑兵，scope=ByType(Cavalry) range=40 下发命令
- **THEN** Toast 消息为"已下发选中骑兵(3)索敌 范围40"

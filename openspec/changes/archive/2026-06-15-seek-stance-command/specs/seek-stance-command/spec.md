## ADDED Requirements

### Requirement: Action::SetSeekStance 命令定义

`Action` 枚举 SHALL 新增 `SetSeekStance` 变体，包含 `scope: SeekScope`、`seek_range: u32`、`unit_ids: Vec<UnitId>` 三个字段。`SeekScope` 枚举 SHALL 包含 `All`、`ByType(SoldierType)` 两个变体。当 `scope` 为 `All` 或 `ByType` 时 `unit_ids` 为空；当需要按选区下发时使用 `unit_ids` 字段。

#### Scenario: 创建全局全体索敌命令

- **WHEN** 构造 `Action::SetSeekStance { scope: All, seek_range: 10, unit_ids: [] }`
- **THEN** 命令合法，可推入 `CommandBuffer`

#### Scenario: 创建按兵种索敌命令

- **WHEN** 构造 `Action::SetSeekStance { scope: ByType(Archer), seek_range: 50, unit_ids: [] }`
- **THEN** 命令合法

#### Scenario: 创建选区索敌命令

- **WHEN** 构造 `Action::SetSeekStance { scope: All, seek_range: 30, unit_ids: [UnitId(7), UnitId(9)] }`
- **THEN** 命令合法（scope 仅用于表示指令类型，unit_ids 决定实际生效单位）

### Requirement: consume_commands 支持 SetSeekStance

`consume_commands_system` SHALL 处理 `Action::SetSeekStance`。消费逻辑 SHALL 按 scope 分类：
- `All` / `ByType`：更新 `GlobalSeekDirective`，并遍历所有匹配的己方单位更新其 `SeekStance`
- `unit_ids` 非空：仅更新指定 UnitId 的 `SeekStance`，不修改 `GlobalSeekDirective`

#### Scenario: 消费全局全体索敌命令

- **WHEN** 消费 `SetSeekStance { scope: All, seek_range: 10, unit_ids: [] }`
- **THEN** `GlobalSeekDirective` 新增一条 `{ scope: All, seek_range: 10, issue_tick: current_tick }` 指令
- **AND** 所有 `Faction::Player` 的士兵 `SeekStance` 更新为 `{ active: true, seek_range: 10 }`

#### Scenario: 消费按兵种索敌命令

- **WHEN** 消费 `SetSeekStance { scope: ByType(Infantry), seek_range: 20, unit_ids: [] }`
- **THEN** `GlobalSeekDirective` 新增一条对应指令
- **AND** 所有 `Faction::Player` 且 `SoldierType == Infantry` 的士兵 `SeekStance` 更新为 `{ active: true, seek_range: 20 }`

#### Scenario: 消费选区索敌命令

- **WHEN** 消费 `SetSeekStance { seek_range: 60, unit_ids: [UnitId(7), UnitId(9)] }`
- **THEN** 仅 UnitId(7) 和 UnitId(9) 的 `SeekStance` 更新为 `{ active: true, seek_range: 60 }`
- **AND** 其他单位不受影响
- **AND** `GlobalSeekDirective` 不修改

#### Scenario: 索敌范围设为 0 等于关闭

- **WHEN** 消费 `SetSeekStance { scope: All, seek_range: 0, unit_ids: [] }`
- **THEN** 所有己方单位 `SeekStance = { active: false, seek_range: 0 }`

### Requirement: UI 输入生成 SetSeekStance 命令

`render_view` 层 SHALL 提供生成 `Action::SetSeekStance` 命令的入口。全局指令通过 HUD 面板输入（范围默认 10）；选区指令通过选中单位后右键/快捷键输入（范围默认 30）。

#### Scenario: 玩家通过全局面板下发全体索敌

- **WHEN** 玩家在索敌面板选择「全体」，输入范围 15，点击确认
- **THEN** 系统生成 `GameCommand { action: SetSeekStance { scope: All, seek_range: 15, unit_ids: [] }, tick: next_tick, player_id: 0 }` 并推入 `CommandBuffer`

#### Scenario: 玩家对选中单位下发索敌

- **WHEN** 玩家选中 3 个骑兵，通过快捷键下发索敌范围 40
- **THEN** 系统生成 `GameCommand { action: SetSeekStance { scope: All, seek_range: 40, unit_ids: [选中的3个UnitId] } }` 并推入 `CommandBuffer`

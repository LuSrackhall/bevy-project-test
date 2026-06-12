## ADDED Requirements

### Requirement: GlobalSeekDirective 资源定义

系统 SHALL 维护 `GlobalSeekDirective` 资源，记录当前生效的索敌指令。该资源包含一个 `Vec<SeekDirective>`，每条指令包含 `scope: SeekScope`（`All` 或 `ByType(SoldierType)`）、`seek_range: u32`、`issue_tick: u32`。资源默认 `Vec::new()`。

#### Scenario: 初始状态无全局指令

- **WHEN** 游戏启动初始化 simulation world
- **THEN** `GlobalSeekDirective.0` 为空 Vec

#### Scenario: 下发全局指令后资源更新

- **WHEN** 消费 `SetSeekStance { scope: All, seek_range: 10 }` 命令
- **THEN** `GlobalSeekDirective.0` 包含该指令，`issue_tick` 等于命令的 `tick`

### Requirement: 新生成单位继承全局指令

`city_spawn_system` 在创建新士兵时 SHALL 查询 `GlobalSeekDirective`。若存在匹配该兵种的指令（`All` 或 `ByType(type)`），SHALL 将新单位的 `SeekStance` 设置为 `{ active: true, seek_range }`。取最后匹配的指令（按 `issue_tick` 排序）。

#### Scenario: 全局 All 指令覆盖新单位

- **WHEN** `GlobalSeekDirective` 包含 `{ scope: All, seek_range: 10 }`，城池产出新民兵
- **THEN** 新民兵 `SeekStance = { active: true, seek_range: 10 }`

#### Scenario: 按兵种指令仅覆盖对应兵种

- **WHEN** `GlobalSeekDirective` 包含 `{ scope: ByType(Archer), seek_range: 50 }`，城池产出新步兵
- **THEN** 新步兵 `SeekStance = { active: false, seek_range: 0 }`（不受弓兵指令影响）

#### Scenario: 多条指令按最后匹配

- **WHEN** `GlobalSeekDirective` 包含 `[{ scope: All, seek_range: 10, issue_tick: 5 }, { scope: ByType(Archer), seek_range: 50, issue_tick: 8 }]`，城池产出新弓兵
- **THEN** 新弓兵 `SeekStance = { active: true, seek_range: 50 }`（按兵种指令后下发，覆盖全局）

### Requirement: 全局指令覆盖规则

后下发的全局指令 SHALL 覆盖先下发的同 scope 指令。不同 scope 的指令 SHALL 共存。`All` 和 `ByType(X)` 之间按 `issue_tick` 确定优先级（后下发覆盖先下发，仅影响匹配的单位）。

#### Scenario: 后下发的 All 覆盖先前 All

- **WHEN** 先消费 `SetSeekStance { scope: All, seek_range: 10 }`，后消费 `SetSeekStance { scope: All, seek_range: 20 }`
- **THEN** 所有己方单位的 `SeekStance.seek_range = 20`

#### Scenario: ByType 与 All 共存

- **WHEN** 先消费 `SetSeekStance { scope: All, seek_range: 10 }`，后消费 `SetSeekStance { scope: ByType(Cavalry), seek_range: 60 }`
- **THEN** 骑兵 `SeekStance.seek_range = 60`，其余兵种 `seek_range = 10`

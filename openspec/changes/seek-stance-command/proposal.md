## Why

当前士兵拥有被动「主动索敌」行为（由 `aggression_range` 配置控制），士兵会自动向范围内的敌人移动并发起攻击。这导致士兵可能违背玩家指令擅自行动，且玩家无法干预。此行为应从默认被动属性改为玩家主动下发的状态命令，确保「无命令不行动」的控制原则。

## What Changes

- **移除** `SoldierUnitConfig.aggression_range` 字段及 `content/units.ron` 中 4 个兵种的该配置项。**BREAKING**（content 配置 schema 变更）
- **新增** `SeekStance` 组件（`active: bool, seek_range: u32`），挂在每个士兵上，默认 `active: false`
- **新增** `GlobalSeekDirective` 资源，记录玩家下发的全局/按兵种索敌指令
- **新增** `Action::SetSeekStance` 命令，支持全局全体、全局按兵种、选中单位三种粒度
- **修改** `combat_engagement_system`：从读取 `aggression_range` 改为读取 `SeekStance`，仅在 `active && 敌人在 seek_range 内` 时执行主动移动
- **保留** 攻击范围内的自动攻击行为不变（`melee_attack_system` / `archer_attack_system` 不受影响）
- **新增** `city_spawn_system` 中新生成单位继承全局 `SeekStance` 的逻辑
- **新增** UI 入口（全局索敌面板 + 选区索敌下发）

## Capabilities

### New Capabilities

- `seek-stance`: 每个士兵携带的 SeekStance 组件，控制是否主动索敌及索敌范围。默认关闭（不主动移动），由玩家指令或全局指令激活。
- `global-seek-directive`: 全局索敌指令资源，记录玩家下发的全局/按兵种指令。新生成单位据此继承初始 SeekStance。
- `seek-stance-command`: GameCommand 流水线中的 SetSeekStance 动作，支持三种粒度（All / ByType / Selection），后下发指令覆盖先前指令。

### Modified Capabilities

- `content-config`: 移除 SoldierUnitConfig 中的 `aggression_range` 字段。units.ron 中 militia/infantry/archer/cavalry 的该行删除。

## Impact

- **simulation/**: `soldier/mod.rs`（SeekStance 组件、combat_engagement_system 修改、city_spawn_system 修改）、`command.rs`（新增 Action 变体、GlobalSeekDirective 资源）、`soldier/config.rs`（移除 aggression_range 字段，添加 serde default）
- **content/**: `units.ron`（删除 aggression_range 行）
- **render_view/**: 新增全局索敌 UI 面板、`selection.rs` 新增选区索敌命令下发
- **AI**: 不受影响——AI 通过直接设置 Movement.target 来操控单位，不依赖 SeekStance

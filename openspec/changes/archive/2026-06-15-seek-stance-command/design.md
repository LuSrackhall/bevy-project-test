## Context

当前系统中，「主动索敌」和「自动攻击」两个概念混在 `combat_engagement_system` 中。该系统的行为由 `SoldierUnitConfig.aggression_range`（被动配置属性）驱动：士兵自动检测 `aggression_range` 范围内的敌人，将 `Movement.target` 设为该敌人，然后 `soldier_movement_system` 驱动移动。玩家对此无法干预。

本设计将主动索敌从被动属性改为玩家可控的状态命令，遵循「无命令不行动」原则。自动攻击（攻击范围内自动打击敌人）保持不变。

约束：
- 架构宪法 CLUA.md：simulation 层必须纯净化，所有输入必须通过 GameCommand 流水线
- 所有空间数值使用 Fixed/i64，禁止 f32
- 确定性要求：同一输入序列必须可复现

## Goals / Non-Goals

**Goals:**
- 移除士兵默认的被动主动索敌行为
- 提供 `SeekStance` 组件，每士兵可独立控制索敌开关及范围
- 提供全局索敌指令（全体/按兵种）与选区指令（按 UnitId）两种粒度
- 后下发的指令可覆盖先前的指令（按单位粒度）
- 新生成的单位自动继承当前全局索敌指令
- 保留攻击范围内的自动攻击行为不变

**Non-Goals:**
- 不修改 AI 行为——AI 通过直接设置 `Movement.target` 操控单位，不依赖 `SeekStance`
- 不改变 melee_attack / archer_attack 系统
- 不实现复杂的 UI（全局面板本期采用最简实现）
- 不处理联机同步——同步复用现有 CommandBuffer 机制即可

## Decisions

### D1: SeekStance 作为组件挂载在每士兵上

每士兵携带 `SeekStance { active: bool, seek_range: u32 }`，默认 `{ false, 0 }`。`combat_engagement_system` 读取此组件决定是否主动移动。

**替代方案：** 用单一全局资源 + 例外列表。拒绝原因：全局资源无法简洁表达「79 号步兵索敌范围 60，其余步兵 20」的混合状态。组件模型天然支持按单位粒度的覆盖。

### D2: 全局指令用 GlobalSeekDirective 资源记录

`GlobalSeekDirective` 资源保存「最近下发的全局指令」，供新生成单位查询继承。命令消费时，遍历现有单位更新组件。

**替代方案：** 无全局资源，仅下发时批量更新。拒绝原因：新生成单位（city_spawn_system）需要知道「当前生效的全局索敌策略」。

### D3: 覆盖语义

后下发指令覆盖先下发指令，按单位粒度生效。不维护优先级回退链。

具体规则：
- 全局 All(range=10) 后 全局 ByType(弓兵, range=50) → 弓兵用 50，其余用 10
- 全局 ByType(步兵, range=20) 后 选区(步兵#7,8,9, range=60) → 7/8/9 用 60，其余步兵用 20
- 选区(弓兵#1, range=30) 后 全局 ByType(弓兵, range=40) → 包括#1 在内的所有弓兵都用 40

实现：全局/按兵种命令消费时遍历所有匹配单位覆盖 `SeekStance`。选区命令直接覆盖指定 UnitId。

**替代方案：** 维护优先级回退链（选区 → 兵种 → 全局）。拒绝原因：增加复杂度和内存开销，且 RTS 场景下「后下发即意图覆盖」更符合玩家预期。

### D4: 默认 seek_range 值

- 全局指令：默认 `seek_range = 10`
- 选区指令：默认 `seek_range = 30`

选区默认更大的原因：玩家精确选中单位通常有明确的战术意图，给予更大初始范围减少二次调整。

### D5: 移除 aggression_range 的处理

从 `SoldierUnitConfig` 中移除 `aggression_range` 字段。添加 `#[serde(default)]` 确保旧配置文件不报错（缺失字段使用默认值 0）。

## Risks / Trade-offs

- **Risk:** 移除 aggression_range 后，现有存档/回放中假设士兵会主动索敌的 AI 行为可能受影响。
  → **Mitigation:** AI 系统直接设置 Movement.target，不依赖 aggression_range。`combat_engagement_system` 中的 `force_move` 守卫（line 75）已确保强制执行命令的单位不被覆盖。

- **Risk:** 全局指令消费时需遍历所有兵种单位（O(n)），在万人同屏时可能有性能影响。
  → **Mitigation:** 此遍历仅发生在玩家手动下发指令时（低频操作，数秒一次），不在每 Tick 执行。不影响常规模拟 Tick 性能。

- **Trade-off:** 覆盖语义下，选区指令可能很快被后续全局指令覆盖，玩家可能困惑。
  → 这是预期行为。UI 上可显示当前生效的全局索敌状态，帮助玩家理解。

## Why

弓箭手箭矢目前无视城池建筑的碰撞，直接穿过城墙击中后方单位。这不符合现实直觉——箭矢理应被实体建筑阻挡。同时，箭矢对建筑的零星攻击不该有显著破坏力（项目已有独立的"士兵自杀式攻城"机制），但完全无碰撞让城池在视觉和逻辑上失去了物理存在感。

## What Changes

- 箭矢飞行阶段新增城池/建筑碰撞检测，使用 `CityRadius` 作为碰撞半径
- 命中建筑时以 1/200 比例计算伤害，通过 `CityComponent` 新增的整数累积器 (`arrow_damage_acc: u32`) 实现精确到小数点后 2 位的伤害
- 建筑碰撞后的箭矢强制进入衰减状态，不触发穿透
- 穿透兵种后的箭矢在同一 Tick 内继续检查建筑碰撞
- 己方箭矢直接穿过己方城池，不发生碰撞
- 战斗全局配置新增 `arrow_building_damage_ratio: 0.005`

## Capabilities

### New Capabilities

- `arrow-building-collision`: 箭矢对城池建筑的碰撞检测、伤害计算与行为规则

### Modified Capabilities

- `archer-directional-arrow`: 箭矢飞行阶段碰撞检测部分，从"仅检测敌方单位"扩展为"同时检测敌方单位与敌方城池建筑"

## Impact

- **simulation/crates/soldier/mod.rs**: `CityComponent` 新增 `arrow_damage_acc: u32` 字段
- **simulation/crates/combat/mod.rs**: `arrow_movement_system` 新增城池碰撞检测逻辑
- **simulation/crates/combat/config.rs**: `CombatGlobalConfig` 新增 `arrow_building_damage_ratio: f32`
- **content/combat.ron**: 新增配置字段

## Why

当前弓箭手箭矢系统存在两个严重问题：1) 箭矢寿命过短（仅 5 Tick / 0.25秒），肉眼无法看到飞行中的箭矢；2) 箭矢采用跟踪目标制导逻辑，不符合弹道物理直觉。此外，弓箭手攻击距离固定 600px，缺乏随等级成长的机制。本 redesign 将箭矢改为固定方向发射 + 碰撞检测 + 穿透机制，并引入等级相关的攻击距离成长。

## What Changes

- **BREAKING**: `Arrow` 组件重构——从跟踪目标制导改为固定方向飞行 + 碰撞检测模型
- **BREAKING**: 删除 `arrow_hit_system`，碰撞逻辑合并入新增的 `arrow_movement_system`
- 新增箭矢生命周期：飞行阶段（可碰撞伤害）→ 衰减阶段（1秒纯视觉，无伤害）→ 销毁
- 新增箭矢穿透机制：Lv.2 起 5%，每升一级 +2%
- 弓箭手攻击距离改为 380-600，随等级线性增长（Lv.1=380, Lv.4=600）
- 箭矢飞行距离 = 攻击距离 + 超射距离（20-80px，随等级变动）
- 飞行 Tick 数在游戏启动时预计算，基于箭速和最大飞行距离
- **BREAKING**: `content/units.ron` 新增 `arrow_speed` 字段
- 渲染：箭矢在衰减阶段逐渐缩小/透明

## Capabilities

### New Capabilities
- `archer-directional-arrow`: 弓箭手发射固定方向箭矢，飞行过程中碰撞敌方单位造成伤害，支持穿透机制，到达最大距离后进入 1 秒衰减动画
- `archer-range-scaling`: 弓箭手攻击距离 380-600 随等级线性增长，等级相关的超射距离

### Modified Capabilities
<!-- No existing specs to modify -->

## Impact

- `crates/simulation/src/combat/mod.rs`: Arrow 组件、archer_attack_system、arrow_hit_system、arrow_expire_system、arrow_movement_system（新增）
- `crates/simulation/src/soldier/config.rs`: SoldierUnitConfig 新增 arrow_speed 字段
- `content/units.ron`: archer 配置新增 arrow_speed
- `crates/render_view/src/debug_shape.rs`: 箭矢渲染支持衰减动画（缩小）
- `crates/simulation/src/lib.rs`: run_tick 阶段顺序调整

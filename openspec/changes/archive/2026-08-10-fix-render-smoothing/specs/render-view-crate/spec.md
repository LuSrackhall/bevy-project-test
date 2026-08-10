## MODIFIED Requirements

### Requirement: DebugShape 几何体渲染

render_view crate SHALL 提供 `debug_shape` 系统用 Bevy `Gizmos` 渲染所有游戏实体。士兵 SHALL 渲染为彩色圆形（Player=蓝、Enemy=红、Neutral=灰），城池 SHALL 渲染为较大的圆形，箭矢 SHALL 渲染为短线段。bevy 版本 SHALL 为 `0.19`，SHALL NOT 依赖 `bevy_prototype_lyon`。运动图元（士兵、箭矢）的位置 SHALL 按 `TickClock` 的 accumulator 在上一 tick 与当前 tick 位置之间插值渲染，SHALL NOT 直接使用 20Hz sim 快照位置。

#### Scenario: 士兵渲染

- **WHEN** 存在 5 个 `Faction::Player` 士兵渲染实体
- **THEN** 屏幕上显示 5 个蓝色圆形，位置由插值后的位置决定，大小由 `SoldierType` 决定（骑兵 14px 半径，其余 10px 半径）

#### Scenario: 城池渲染

- **WHEN** 城池的 `CityRadius` 为 20 像素半径
- **THEN** 屏幕上显示一个对应颜色的圆形，圆心为插值后的位置，半径为 `CityRadius` 的浮点转换值

#### Scenario: 相邻 tick 间平滑插值

- **WHEN** 单位在 tick N 的位置为 P0、tick N+1 的位置为 P1，且 `TickClock.accumulator` 介于 0 与 `tick_duration` 之间
- **THEN** 渲染位置 SHALL 为 `lerp(P0, P1, accumulator / tick_duration)`

#### Scenario: 新生成实体回退

- **WHEN** 一个实体刚生成、尚无上一 tick 位置记录
- **THEN** 渲染位置 SHALL 回退为其当前 sim 位置（不插值）

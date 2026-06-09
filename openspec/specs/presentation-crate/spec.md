# presentation-crate

## Purpose

TBD

## Requirements


### Requirement: LogicEntityRef 单向引用

presentation crate SHALL 定义 `LogicEntityRef(pub UnitId)` 组件，挂载在渲染实体上指回逻辑实体。逻辑实体 SHALL NOT 持有任何指向渲染实体的引用。

#### Scenario: 渲染实体持有逻辑引用

- **WHEN** bevy_adapter 为新诞生的逻辑单位创建 Bevy 实体
- **THEN** presentation 层为该实体挂载 `LogicEntityRef(unit_id)` 和插值组件

#### Scenario: 仿真层无渲染引用

- **WHEN** 审查 simulation crate 中所有组件和资源定义
- **THEN** SHALL NOT 存在任何指向渲染实体或 `Entity` 的字段

### Requirement: InterpolationData 插值数据

presentation crate SHALL 在每个渲染实体上挂载 `InterpolationData { previous_logical_pos: Vec2, current_logical_pos: Vec2, is_new: bool }` 组件。SHALL 提供 `RenderInterpolationAlpha(pub f32)` 全局资源表示当前帧的插值因子。

#### Scenario: 每帧更新插值历史

- **WHEN** simulation 在 Tick N 完成，单位从 `pos_A` 移动到 `pos_B`
- **THEN** presentation 层的插值更新系统将 `previous_logical_pos` 设为 `pos_A`（浮点），`current_logical_pos` 设为 `pos_B`（浮点）

#### Scenario: 新实体首帧无闪烁

- **WHEN** 一个逻辑单位在 Tick N 中途诞生（如城池产兵）
- **THEN** 该渲染实体的 `InterpolationData.is_new = true`，`previous_logical_pos == current_logical_pos`，`PresentationPosition` 直接等于逻辑位置，不执行插值计算

#### Scenario: 第二帧开始正常插值

- **WHEN** 新实体的 `is_new == true` 且经过第一个逻辑 Tick 后
- **THEN** `is_new` 被设置为 `false`，后续帧正常执行插值计算

### Requirement: PresentationPosition 插值输出

presentation crate SHALL 在每个渲染实体上挂载 `PresentationPosition(pub Vec2)` 组件，存储当前帧的插值后平滑位置。SHALL 使用 `f32`/`Vec2` 浮点数类型（仅限 presentation 和 render_view 层）。

#### Scenario: 线性插值计算

- **WHEN** `InterpolationData.previous_logical_pos = Vec2(0, 0)`，`InterpolationData.current_logical_pos = Vec2(100, 0)`，`RenderInterpolationAlpha = 0.5`
- **THEN** `PresentationPosition = Vec2(50, 0)`

#### Scenario: Alpha 边界值

- **WHEN** `RenderInterpolationAlpha = 0.0`
- **THEN** `PresentationPosition = InterpolationData.previous_logical_pos`

- **WHEN** `RenderInterpolationAlpha = 1.0`
- **THEN** `PresentationPosition = InterpolationData.current_logical_pos`

### Requirement: 实体生灭监听

presentation crate SHALL 监听 `LogicEntityRef` 组件的添加（`Added<LogicEntityRef>`）来初始化插值状态。SHALL 监听 `LogicEntityRef` 组件所在实体的移除来清理渲染实体。

#### Scenario: 监听新绑定实体

- **WHEN** bevy_adapter 为新逻辑实体创建了挂载 `LogicEntityRef(unit_id)` 的 Bevy 实体
- **THEN** presentation 层在下一帧检测到 `Added<LogicEntityRef>`，查询 simulation 的 `LogicalPosition`，将浮点位置写入 `previous_logical_pos` 和 `current_logical_pos`，设置 `is_new = true`

#### Scenario: 渲染实体跟随销毁

- **WHEN** bevy_adapter 因 `UnitDestroyed` 事件销毁了 Bevy 实体
- **THEN** presentation 层不需要额外操作（Bevy 的父子关系自动清理子实体和组件）

### Requirement: RenderInterpolationAlpha 计算

presentation crate SHALL 每帧根据 `TickClock.accumulator` 和 `TickClock.tick_duration` 计算 `RenderInterpolationAlpha`：`alpha = accumulator / tick_duration`，取值范围 [0.0, 1.0)。

#### Scenario: 帧在 Tick 中点

- **WHEN** Tick 间隔 50ms，当前帧的 `accumulator = 25ms`
- **THEN** `RenderInterpolationAlpha = 0.5`

#### Scenario: 帧紧接 Tick 之后

- **WHEN** `accumulator ≈ 1ms`（刚执行完一个 Tick）
- **THEN** `RenderInterpolationAlpha ≈ 0.02`

### Requirement: 浮点数不得回流

presentation crate SHALL 仅在本层和 render_view 层使用浮点数。SHALL NOT 将任何浮点值写回 simulation 或 bevy_adapter 层的资源或组件。

#### Scenario: 编译时类型屏障

- **WHEN** 尝试在 presentation 中修改 simulation 的 `LogicalPosition`（类型 `FixedVec2`）为 `Vec2` 浮点值
- **THEN** 编译失败，因为 `LogicalPosition(FixedVec2)` 不接受 `Vec2`

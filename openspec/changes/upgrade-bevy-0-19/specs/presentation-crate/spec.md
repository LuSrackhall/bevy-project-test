## MODIFIED Requirements

### Requirement: InterpolationData 插值数据

presentation crate SHALL 在每个渲染实体上挂载 `InterpolationData { previous_logical_pos: Vec2, current_logical_pos: Vec2, is_new: bool }` 组件。SHALL 提供 `RenderInterpolationAlpha(pub f32)` 全局资源表示当前帧的插值因子。bevy 版本 SHALL 为 `0.19`。

#### Scenario: 每帧更新插值历史

- **WHEN** simulation 在 Tick N 完成，单位从 `pos_A` 移动到 `pos_B`
- **THEN** presentation 层的插值更新系统将 `previous_logical_pos` 设为 `pos_A`（浮点），`current_logical_pos` 设为 `pos_B`（浮点）

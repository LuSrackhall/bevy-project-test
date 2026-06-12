## Why

单位头顶信息条的血量条和经验条填充矩形始终不可见（血条全红、经验条全蓝）。经排查，`bevy_prototype_lyon` 的 `ShapeBuilder::fill()` 对背景矩形有效，但通过 `commands` despawn/respawn 创建的填充矩形不渲染——可能是 mesh 生成或 parent 绑定存在延迟问题。

将填充条从 `ShapeBundle` 改为 Bevy 原生 `Sprite` 可彻底消除此问题：`Sprite` 是 Bevy 最基础的渲染组件，行为完全可预测，且更新时只需修改 `custom_size` 字段，无需 despawn/respawn。

## What Changes

- 将 `create_bar` 和 `update_bar` 中 HP fill / EXP fill 的创建方式从 `ShapeBuilder::fill().build()` 改为 `Sprite { color, custom_size }`
- 更新逻辑从 despawn+respawn 改为直接修改 `Sprite.custom_size` 和 `Sprite.color`
- 背景条保持 `ShapeBundle` 不变（已正常工作）
- 移除 `HpFill` / `ExpFill` marker 组件（不再需要标记用于查找 fill 实体）
- `BarParts` 中 `hp_fill` / `exp_fill` 字段保留（用于定位 Sprite 实体）

## Capabilities

### New Capabilities

无新增能力。

### Modified Capabilities

无修改需求规格（纯实现层 bug 修复）。

## Impact

- 影响 crate: `render_view/`（仅 `unit_info_bar.rs`）
- 不引入新依赖（`Sprite` 是 Bevy 内置组件）
- 不影响 `simulation`、`bevy_adapter`、`presentation` 层

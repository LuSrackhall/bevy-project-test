## Why

渲染层是当前瓶颈。Simulation 经过两轮优化后 1000 单位战斗 tick = 10.6ms（预算 50ms），但 1500+ 单位仍帧率下降。`unit_info_bar_system` 每帧对每个单位无条件调用 `format!` + 更新 8-11 个子实体组件，产生 ~11000 ECS mutations/帧。

## What Changes

- InfoBar 脏标记：缓存 HP/Level/EXP，值不变时跳过 format! + Text2d 更新
- 视口裁剪：Camera AABB 过滤屏幕外单位，跳过渲染
- 默认 InfoBarMode=Selected：仅选中单位显示 bar

## Capabilities

### New Capabilities
- `render-dirty-tracking`: InfoBar 脏标记缓存
- `render-viewport-culling`: 视口裁剪

### Modified Capabilities
- `render-view-crate`: InfoBarMode 默认值变更

## Impact

- `crates/render_view/src/unit_info_bar.rs` — 主要修改
- `crates/render_view/src/debug_shape.rs` — 删除死代码 _positions HashMap + 视口裁剪
- `crates/render_view/src/camera.rs` — 可能需要导出 viewport AABB 计算
- 无 simulation 层改动

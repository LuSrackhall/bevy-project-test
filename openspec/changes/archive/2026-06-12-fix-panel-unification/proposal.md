## Why

选中逻辑已统一（城池/兵种互斥），但底部布局仍是三区：兵种面板在左 30%，城池面板在中 40%。两者在不同位置，未实现"同一位置展示"的设计意图。

## What Changes

- **布局改为二区**：左 30% 信息面板 + 右 70% 命令卡片+图鉴
- **城池和兵种面板叠在同一左区 Node**，通过 visibility 切换
- **移除中区城池面板、移除右区小地图占位**
- **更新逻辑**：根据选中类型显示对应面板，隐藏另一个

## Capabilities

### Modified Capabilities

- `render-view-crate`: HUD 布局从三区简化为二区

## Impact

- `crates/render_view/src/ui/hud.rs`: 底部布局重排，面板 show/hide 逻辑

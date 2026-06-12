## Why

当前 HUD 仅对城池选中显示底部面板，士兵无任何属性面板。选中反馈极弱，操作不直观。需要建立 RTS 标准的 UI 骨架——用 Node 布局规划三区（信息面板、命令卡片、小地图），后期替换美术贴图即可。

## What Changes

**底部三区骨架（180px）**：
- **左 30% — 选中信息面板**：单选士兵展示 HP/ATK/SPD/RNG/EXP/特殊效果/所属城池；多选聚合展示；城池增强版面板
- **中 40% — 命令卡片**：移动/攻击/停止/驻守按钮 + 选中概要
- **右 30% — 小地图预留 + 兵种图鉴**：小地图框架 + 悬停兵种按钮时显示完整属性描述

**交互增强**：
- 城池面板兵种按钮悬停 → 右区显示该兵种图鉴
- 兵种特殊效果文案（民兵/步兵/弓兵/骑兵各有描述）

**架构**：render_view 层纯 UI，通过 SimulationWorld 读取数据。不引入新依赖。

## Capabilities

### New Capabilities

- `hud-soldier-panel`: 士兵选中信息面板
- `hud-command-card`: 命令卡片区域
- `hud-unit-compendium`: 兵种图鉴悬停显示

### Modified Capabilities

- `render-view-crate`: HUD 布局重构
- `city-interaction`: 城池面板移至左区

## Impact

- `crates/render_view/src/ui/hud.rs`: 重写 setup 和 update 逻辑
- `crates/render_view/src/ui/` (可能新增): 拆分大文件为 panel/soldier_panel/city_panel/command_card/compendium

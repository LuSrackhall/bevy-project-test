## Why

当前游戏只有底部面板（HUD）显示选中单位的等级、血量和经验值。在 RTS 游戏中，玩家需要快速扫视战场获取信息——必须点击单位才能查看其生命值和等级会降低操作效率和战场感知力。在单位头顶直接显示关键信息是 RTS 品类的基础体验。

## What Changes

- 新增 `unit-overhead-info-bar` 模块：在单位（士兵+城市）头顶以世界空间绘制浮动信息条
- 信息条包含：等级数字（Text2D）、血量条（ShapeBundle 矩形）、经验条（ShapeBundle 矩形），均为固定像素尺寸
- 新增三种显示模式（始终显示/仅选中/智能），通过 `Ctrl+H` 循环切换，默认 Smart 模式
- 精简底部面板：移除士兵面板的血量和经验显示，移除城市面板的等级、血量和经验显示，保留攻击/速度/特殊技能/人口/训练类型
- **BREAKING**: 底部面板 UI 布局变更——士兵和城市详细信息中的血量、等级、经验字段将不再显示

## Capabilities

### New Capabilities
- `unit-overhead-info-bar`: 在单位头顶世界空间渲染等级文字、血量条和经验条，支持士兵和城市两种实体类型
- `info-bar-display-modes`: 三种显示模式（Always/Selected/Smart），支持通过 `Ctrl+H` 快捷键循环切换，Smart 为默认模式

### Modified Capabilities
- `ui-system-fixes`: 底部面板精简——士兵面板移除血量和经验显示，城市面板移除等级、血量和经验显示

## Impact

- 影响 crate: `render_view/`（新增 `unit_info_bar.rs`，修改 `hud.rs`、`lib.rs`）
- 新增依赖: 使用项目已有 `bevy_prototype_lyon` 和 `Text2D`，无新外部依赖
- 数据读取: 读取 `Health`、`Level`、`GlobalTransform`、`SelectionState`，均为已有组件/资源
- 不涉及 `simulation`/`bevy_adapter`/`presentation` 层修改
- 快捷键冲突检查: `Ctrl+H` 当前未被占用

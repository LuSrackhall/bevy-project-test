## MODIFIED Requirements

### Requirement: DebugShape 几何体渲染

render_view crate SHALL 提供 `debug_shape` 系统用 Bevy `Gizmos` 渲染所有游戏实体。士兵 SHALL 渲染为彩色圆形（Player=蓝、Enemy=红、Neutral=灰），城池 SHALL 渲染为较大的圆形，箭矢 SHALL 渲染为短线段。bevy 版本 SHALL 为 `0.19`，SHALL NOT 依赖 `bevy_prototype_lyon`。

#### Scenario: 士兵渲染

- **WHEN** 存在 5 个 `Faction::Player` 士兵渲染实体
- **THEN** 屏幕上显示 5 个蓝色圆形，位置由 `PresentationPosition` 决定，大小由 `SoldierType` 决定（骑兵 14px 半径，其余 10px 半径）

#### Scenario: 城池渲染

- **WHEN** 城池的 `CityRadius` 为 20 像素半径
- **THEN** 屏幕上显示一个对应颜色的圆形，圆心为 `PresentationPosition`，半径为 `CityRadius` 的浮点转换值

### Requirement: 选择系统

render_view crate SHALL 提供 `SelectionState` 资源（`selected_unit_ids: Vec<UnitId>`）和选择系统（左键点选、左键拖拽框选/圈选、Ctrl+点选追加、Ctrl+A 全选、Esc 取消选择）。选中指示器和拖拽框视觉 SHALL 使用 Bevy 原生 `Gizmos` API 绘制，SHALL NOT 使用 `bevy_prototype_lyon`。

#### Scenario: 单点选

- **WHEN** 玩家左键点击一个友方士兵
- **THEN** `SelectionState.selected_unit_ids` 替换为仅该士兵的 `UnitId`

#### Scenario: 矩形框选

- **WHEN** 玩家左键拖拽一个矩形区域包围 5 个友方士兵
- **THEN** `SelectionState.selected_unit_ids` 包含这 5 个士兵的 `UnitId`

#### Scenario: 拖拽框视觉反馈

- **WHEN** 玩家正在拖拽选择
- **THEN** 屏幕上使用 `Gizmos::rect_2d`（矩形模式）或 `Gizmos::circle_2d`（圆形模式）显示半透明选择区域

#### Scenario: 选中指示器

- **WHEN** `SelectionState.selected_unit_ids` 包含 3 个 UnitId
- **THEN** 这 3 个渲染实体周围使用 `Gizmos::circle_2d` 显示绿色圆圈指示器

### Requirement: HUD 系统

render_view crate SHALL 提供 HUD，包含：顶部信息栏（城池数/人口数/游戏时间/暂停按钮）、底部城池详情面板（仅在选中友方城池时显示：等级/HP条/人口/经验/兵种按钮）、底部工具栏（圈选框选切换/举盾/强制移动按钮）。bevy 版本 SHALL 为 `0.19`，UI Widget 插件（`ButtonPlugin`、`MenuPlugin` 等）SHALL NOT 手动注册（已含于 `DefaultPlugins`）。`TextFont` 构造 SHALL 使用 `FontSize::Px(...)` 替代裸 `f32`。

#### Scenario: 顶部栏数据更新

- **WHEN** 玩家占领一座新城池
- **THEN** 顶部栏 "城 X/Y" 更新为新的城池计数

#### Scenario: 文字正常显示

- **WHEN** 进入游戏，各 UI 面板的文字元素加载完成
- **THEN** 所有文字（等级、HP数值、EXP数值、按钮标签）正常显示，字体大小符合预期

### Requirement: 主菜单系统

主菜单 SHALL 提供 4 个地图大小按钮（小/中/大/巨大），点击后设 `NeedsGameReset::NewGame(MapSize)` 并进入 `Playing`。`TextFont` 构造 SHALL 使用 `FontSize::Px(...)` 替代裸 `f32`。

#### Scenario: 选择 Huge 地图

- **WHEN** 用户点击"巨大"
- **THEN** `NeedsGameReset` 设为 `NewGame(MapSize::Huge)`，`GameState` 切换到 `Playing`

### Requirement: 暂停菜单系统

render_view crate SHALL 提供暂停菜单（半透明遮罩、"继续游戏"/"重新开始"/"返回主菜单"按钮），在 `GameState::Paused` 时显示。按 Esc 键从 Playing 切换到 Paused（若无选区或有选区时先清除选区）。`TextFont` 构造 SHALL 使用 `FontSize::Px(...)` 替代裸 `f32`。

#### Scenario: 暂停后继续

- **WHEN** 玩家在暂停菜单点击"继续游戏"
- **THEN** `GameState` 切换回 `Playing`

### Requirement: 结算画面系统

render_view crate SHALL 提供结算画面，在 `GameState::GameOver` 时显示。展示胜利/失败文本、游戏时长、剩余城池数、总击杀数、"再来一局"/"返回主菜单"按钮。`TextFont` 构造 SHALL 使用 `FontSize::Px(...)` 替代裸 `f32`。

#### Scenario: 玩家胜利

- **WHEN** 所有敌方城池被消灭，`check_victory_system` 检测到 `enemy_cities.is_empty()`
- **THEN** `GameState` 切换到 `GameOver`，结算画面显示"胜利!"

#### Scenario: 玩家失败

- **WHEN** 所有玩家城池被消灭
- **THEN** 结算画面显示"失败!"

### Requirement: 血条信息栏

render_view crate SHALL 提供 `UnitInfoBar` 系统，为每个存活单位显示等级文字、血条（HP背景 + HP填充）、经验条（EXP背景 + EXP填充）、护盾条（可选）。血条背景矩形 SHALL 使用 `Sprite { color, custom_size }` 实现，SHALL NOT 使用 `bevy_prototype_lyon`。`TextFont` 构造 SHALL 使用 `FontSize::Px(...)` 替代裸 `f32`。

#### Scenario: 血条背景渲染

- **WHEN** 一个 HP 为 80/100 的单位被选中
- **THEN** 血条背景为红色 `Sprite`，HP 填充为绿色 `Sprite`（宽度按比例），无第三方依赖

#### Scenario: 护盾条渲染

- **WHEN** 单位装备了护盾（`shield_max > 0`）
- **THEN** 显示护盾条背景（灰色 `Sprite`）和护盾填充（白色 `Sprite`），以及护盾数值文字

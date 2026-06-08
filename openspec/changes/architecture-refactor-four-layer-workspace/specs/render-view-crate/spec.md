## ADDED Requirements

### Requirement: DebugShape 几何体渲染

render_view crate SHALL 提供 `debug_shape` 系统用 Bevy `Gizmos` 渲染所有游戏实体。士兵 SHALL 渲染为彩色圆形（Player=蓝、Enemy=红、Neutral=灰），城池 SHALL 渲染为较大的圆形，箭矢 SHALL 渲染为短线段。

#### Scenario: 士兵渲染

- **WHEN** 存在 5 个 `Faction::Player` 士兵渲染实体
- **THEN** 屏幕上显示 5 个蓝色圆形，位置由 `PresentationPosition` 决定，大小由 `SoldierType` 决定（骑兵 14px 半径，其余 10px 半径）

#### Scenario: 城池渲染

- **WHEN** 城池的 `CityRadius` 为 20 像素半径
- **THEN** 屏幕上显示一个对应颜色的圆形，圆心为 `PresentationPosition`，半径为 `CityRadius` 的浮点转换值

#### Scenario: 箭矢渲染

- **WHEN** 弓兵向敌方发射箭矢
- **THEN** 箭矢在仿真层为逻辑实体，渲染层以直线线段或小圆形表示其飞行路径的当前段

### Requirement: 相机系统

render_view crate SHALL 提供相机系统，包括主相机（`Camera2d`）、中键/右键拖拽漫游、滚轮缩放。相机 SHALL 在游戏开始时定位到第一个玩家城池。

#### Scenario: 中键拖拽

- **WHEN** 玩家按住鼠标中键并拖动
- **THEN** 相机 `Transform.translation` 随鼠标移动方向平移

#### Scenario: 滚轮缩放

- **WHEN** 玩家滚动鼠标滚轮
- **THEN** 相机 `OrthographicProjection.scale` 在 [0.2, 3.0] 范围内改变，缩放速度与滚轮增量成正比

#### Scenario: 初始定位

- **WHEN** 游戏开始，地图生成完毕后
- **THEN** 相机中心自动移动到第一个 `Faction::Player` 城池的位置

### Requirement: 选择系统

render_view crate SHALL 提供 `SelectionState` 资源（`selected_unit_ids: Vec<UnitId>`）和选择系统（左键点选、左键拖拽框选/圈选、Ctrl+点选追加、Ctrl+A 全选、Esc 取消选择）。

#### Scenario: 单点选

- **WHEN** 玩家左键点击一个友方士兵
- **THEN** `SelectionState.selected_unit_ids` 替换为仅该士兵的 `UnitId`

#### Scenario: 矩形框选

- **WHEN** 玩家左键拖拽一个矩形区域包围 5 个友方士兵
- **THEN** `SelectionState.selected_unit_ids` 包含这 5 个士兵的 `UnitId`

#### Scenario: 圆形框选

- **WHEN** `SelectionState.selection_mode == Circle` 且玩家左键拖拽
- **THEN** 以拖拽起点为圆心、拖拽距离为半径，选中半径内所有友方士兵

#### Scenario: Ctrl+点选追加

- **WHEN** 玩家按住 Ctrl 并点击一个未选中的友方士兵
- **THEN** 该士兵的 `UnitId` 被追加到 `SelectionState.selected_unit_ids` 中

#### Scenario: Ctrl+A 全选

- **WHEN** 玩家按下 Ctrl+A（或 Cmd+A）
- **THEN** `SelectionState.selected_unit_ids` 包含所有 `Faction::Player` 的士兵 UnitId

#### Scenario: 拖拽框视觉反馈

- **WHEN** 玩家正在拖拽选择
- **THEN** 屏幕上显示半透明矩形（`SelectionMode::Rect`）或圆形（`SelectionMode::Circle`）选择区域

#### Scenario: 选中指示器

- **WHEN** `SelectionState.selected_unit_ids` 包含 3 个 UnitId
- **THEN** 这 3 个渲染实体周围显示绿色圆圈指示器

### Requirement: HUD 系统

render_view crate SHALL 提供 HUD，包含：顶部信息栏（城池数/人口数/游戏时间/暂停按钮）、底部城池详情面板（仅在选中友方城池时显示：等级/HP条/人口/经验/兵种按钮）、底部工具栏（圈选框选切换/举盾/强制移动按钮）。

#### Scenario: 顶部栏数据更新

- **WHEN** 玩家占领一座新城池
- **THEN** 顶部栏 "城 X/Y" 更新为新的城池计数

#### Scenario: 选中城池显示底部面板

- **WHEN** 玩家左键点击友方城池
- **THEN** 底部面板从 `Display::None` 变为 `Display::Flex`，显示该城池的等级、HP 条、人口、经验、兵种切换按钮

#### Scenario: 点击空地隐藏底部面板

- **WHEN** 底部面板处于显示状态，玩家点击空地
- **THEN** 底部面板变为 `Display::None`

#### Scenario: 兵种切换按钮

- **WHEN** 选中友方城池，玩家点击底部面板的"骑兵"按钮
- **THEN** 通过 `bevy_adapter` 发出 `Action::SetSpawnType { city: ..., soldier_type: Cavalry }` 命令，城池后续产兵类型变为骑兵

#### Scenario: 举盾按钮

- **WHEN** 选中了至少一个步兵 UnitId，玩家点击"盾"按钮
- **THEN** 对每个选中步兵发出 `Action::SetShield { unit: ..., state: ShieldUp/Normal }` 命令

#### Scenario: 强制移动按钮

- **WHEN** 玩家点击"优先"（强制移动）按钮
- **THEN** `ForceMoveNext.active` 设置为 `true`，下一次右键命令将发出 `Action::ForceMove` 而非 `Action::MoveTo`

### Requirement: 主菜单系统

render_view crate SHALL 提供主菜单界面（游戏标题、"单人模式"按钮等），在 `GameState::MainMenu` 时显示。点击"单人模式"进入 `GameState::Playing`。

#### Scenario: 进入游戏

- **WHEN** 玩家在主菜单点击"单人模式"
- **THEN** `GameState` 切换到 `Playing`，主菜单 UI 清除，游戏世界开始初始化

### Requirement: 暂停菜单系统

render_view crate SHALL 提供暂停菜单（半透明遮罩、"继续游戏"/"重新开始"/"返回主菜单"按钮），在 `GameState::Paused` 时显示。按 Esc 键从 Playing 切换到 Paused（若无选区或有选区时先清除选区）。

#### Scenario: 暂停后继续

- **WHEN** 玩家在暂停菜单点击"继续游戏"
- **THEN** `GameState` 切换回 `Playing`

#### Scenario: Esc 行为

- **WHEN** 当前有选中的士兵，玩家按 Esc
- **THEN** 选区被清除（不进入暂停）。再次按 Esc 才进入暂停

### Requirement: 结算画面系统

render_view crate SHALL 提供结算画面，在 `GameState::GameOver` 时显示。展示胜利/失败文本、游戏时长、剩余城池数、总击杀数、"再来一局"/"返回主菜单"按钮。

#### Scenario: 玩家胜利

- **WHEN** 所有敌方城池被消灭，`check_victory_system` 检测到 `enemy_cities.is_empty()`
- **THEN** `GameState` 切换到 `GameOver`，结算画面显示"胜利!"

#### Scenario: 玩家失败

- **WHEN** 所有玩家城池被消灭
- **THEN** 结算画面显示"失败!"

### Requirement: 渲染层不写回仿真状态

render_view crate SHALL NOT 直接修改任何仿真层定义的数据（`LogicalPosition`、`Health`、`CityComponent` 等）。所有影响游戏状态的用户操作 SHALL 通过 `CommandBuffer` 命令管道。

#### Scenario: 编译时隔离

- **WHEN** 审查 `render_view/src/` 中所有文件的 import 语句
- **THEN** SHALL NOT 包含对 simulation crate 组件的 `mut` 引用（Query 或 ResMut）

# ui-system-fixes

## Purpose

UI 系统修复与改进：顶部状态栏、暂停按钮、底部面板、HP 条、兵种切换等 HUD 功能。

## Requirements

### Requirement: Top status bar updates in real time
顶部状态栏 SHALL 每帧显示最新数据：玩家城池数/总城池数、玩家总人口、已运行时间（mm:ss 格式）。HUD 更新系统 SHALL 仅在 `GameState::Playing` 且 `Paused.0 == false` 时运行。

#### Scenario: Status bar shows player stats
- **WHEN** 游戏 Playing 状态中且未暂停，玩家拥有 3 座城池、总计 25 人口
- **THEN** 顶部状态栏显示 "城 3/N"（N 为总城池数）、"兵 25"、"T MM:SS"（已运行时间）

#### Scenario: 暂停时不更新
- **WHEN** `GameState::Playing` 且 `Paused.0 == true`
- **THEN** HUD 更新系统不运行

### Requirement: Pause button in top bar works
顶部状态栏的暂停按钮 SHALL 点击时设 `Paused(true)`，SHALL NOT 切换 `GameState`。

#### Scenario: Click pause button
- **WHEN** 玩家点击顶部状态栏的暂停按钮
- **THEN** `Paused` 设为 `true`，暂停菜单 UI 可见性切换为 `Visible`，`GameState` 不变

### Requirement: Bottom panel shows selected city data
选中己方城池后，底部面板 SHALL 显示城池的实时数据（等级、血量血条、人口）。取消选中后面板隐藏。

#### Scenario: Select player city
- **WHEN** 玩家选中一座己方城池 Lv.3，HP 200/300，人口 15/45
- **THEN** 底部面板显示：`[城池] Lv.3 (上限 7) | 血量条 (200/300) | 兵 15/45`

#### Scenario: Deselect city
- **WHEN** 玩家点击空地取消城池选中
- **THEN** 底部面板隐藏（display: None）

### Requirement: HP bar rendered as Bevy Node
底部面板的血量指示 SHALL 使用 Bevy UI Node 渲染（灰色底色 + 绿色/红色填充条，宽度按 HP 百分比），不使用 emoji 或 text 模拟。

#### Scenario: HP at 67%
- **WHEN** 城池 HP 为 200/300
- **THEN** 血条填充 Node 宽度为容器的 67%，颜色为绿色（>50%）或红色（≤50%）

### Requirement: Soldier type buttons change spawn type
底部面板的 4 个兵种按钮 SHALL 点击时将当前选中城池的 spawn_type 切换为对应兵种，并高亮当前选中兵种。

#### Scenario: Switch to cavalry
- **WHEN** 玩家选中己方城池，点击 "骑兵" 按钮
- **THEN** 该城池 spawn_type 变为 Cavalry，"骑兵" 按钮背景高亮，其他按钮恢复普通状态

### Requirement: Shield toggle button works
工具栏的举盾按钮 SHALL 切换所有选中步兵的 InfantryShield 状态。若选中步兵中有举盾和正常混合状态，统一变为举盾（再一次点击全部正常）。

#### Scenario: Mixed shield states toggle to all shield-up
- **WHEN** 玩家选中 3 个步兵（2 个举盾、1 个正常），点击 "盾" 按钮
- **THEN** 全部 3 个步兵变为举盾状态

#### Scenario: All shield-up toggle to all normal
- **WHEN** 玩家选中 3 个步兵（全部举盾），点击 "盾" 按钮
- **THEN** 全部 3 个步兵变为正常状态

### Requirement: All UI text uses plain text or Node rendering
所有 UI 文字 SHALL 使用纯文本（中文/英文）或 Bevy Node 渲染。不使用任何 emoji 字符作为图标。游戏标题 "⚔️ 城池争霸" 替换为 "城池争霸"。

#### Scenario: Main menu renders correctly
- **WHEN** 游戏启动进入主菜单
- **THEN** 标题显示 "城池争霸"（纯文字），按钮显示 "单人模式"、"多人模式 (开发中)"、"设置"、"帮助"

### Requirement: SeekScopeDropdown popup appears without position flicker
SeekScopeDropdown 的弹出面板 SHALL 在展开时直接出现在正确位置，无位置跳变或闪烁。实现方式为在 observer 中手动计算 popup 的 `top` 位置（基于 anchor 的 `UiGlobalTransform` 和窗口尺寸），不使用 Bevy 的 Popover 系统。

#### Scenario: First open shows popup at correct position
- **WHEN** 玩家点击 SeekScopeDropdown 按钮触发下拉菜单展开
- **THEN** popup 面板首次出现时已在正确位置（按钮上方或下方），无任何位置跳变或闪烁

#### Scenario: Popup position adapts to available space
- **WHEN** 触发按钮靠近窗口底部，上方空间不足以容纳 popup
- **THEN** popup 出现在按钮下方；否则优先出现在按钮上方

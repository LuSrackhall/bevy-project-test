## MODIFIED Requirements

### Requirement: HUD 布局

render_view crate SHALL 提供 HUD，包含：顶部信息栏（城池数/人口数/游戏时间/暂停按钮）、游戏画面区域、**底部三区面板**（左 30% 选中信息面板、中 40% 命令卡片、右 30% 小地图预留+兵种图鉴）。所有面板使用 Bevy UI Node 骨架布局，后期替换美术贴图。HUD 代码 SHALL 拆分为多个模块文件（soldier_panel / city_panel / command_card / compendium）。

#### Scenario: 选中城池显示底部面板

- **WHEN** 玩家左键点击友方城池
- **THEN** 底部左区显示城池信息面板，中区显示城池命令，右区显示小地图

#### Scenario: 选中士兵显示属性面板

- **WHEN** 玩家框选士兵
- **THEN** 底部左区显示士兵属性面板（单选全属性，多选聚合），中区显示移动攻击命令

#### Scenario: 悬停兵种按钮查看图鉴

- **WHEN** 玩家鼠标悬停在城池面板的兵种按钮上
- **THEN** 右区兵种图鉴切换显示该兵种的完整属性与描述

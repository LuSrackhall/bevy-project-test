## ADDED Requirements

### Requirement: Overhead info bar for soldiers
系统 SHALL 在每个士兵单位头顶上方渲染信息条，包含等级文字、血量条和经验条，采用世界空间锚定。

#### Scenario: Soldier with full health and no experience
- **WHEN** 战场上存在一个 Lv.1、HP 100/100、EXP 0/50 的士兵
- **THEN** 该士兵头顶显示 "Lv.1" 白色文字、全满绿色血条（40×6px）、空进度紫色经验条（40×4px）

#### Scenario: Soldier with partial health and experience
- **WHEN** 战场上存在一个 Lv.2、HP 45/80、EXP 30/60 的士兵
- **THEN** 该士兵头顶显示 "Lv.2" 白色文字、绿色填充宽度为血条 56.25% 的血条、紫色填充宽度为经验条 50% 的经验条

#### Scenario: Soldier dies
- **WHEN** 一个有信息条的士兵被击杀
- **THEN** 其信息条子实体随士兵实体一同销毁，无残留

### Requirement: Overhead info bar for cities
系统 SHALL 在每个城市单位头顶上方渲染信息条，包含等级文字、血量条和经验条，与士兵使用相同的视觉规格。

#### Scenario: City with full health
- **WHEN** 战场上存在一个 Lv.3、HP 300/300、EXP 80/100 的城市
- **THEN** 该城市头顶显示 "Lv.3" 白色文字、全满绿色血条、紫色填充宽度为 80% 的经验条

### Requirement: Info bar visual layout
信息条 SHALL 采用层次叠加布局：血条在最上（粗，6px 高），经验条在下（细，4px 高），等级文字在血条左上方，数值文字在条右侧叠加。

#### Scenario: Visual element positions
- **WHEN** 一个单位有信息条
- **THEN** 等级文字位于单位头顶上方 12px 处、水平居中偏左；血条在等级文字下方 2px；经验条在血条下方 2px

### Requirement: Info bar color scheme
信息条 SHALL 使用经典 RTS 配色：血条红色底（#CC0000）+ 绿色填充（#00CC00），经验条蓝色底（#2244AA）+ 紫色填充（#AA44FF），等级和数值文字白色（#FFFFFF）。

#### Scenario: HP bar color at different health levels
- **WHEN** 单位 HP 为 80%
- **THEN** 血条填充部分为绿色 #00CC00，宽度为总血条宽度的 80%

### Requirement: Health and experience numeric overlay
血条和经验条右侧 SHALL 叠加显示 "当前值/最大值" 的白色数字（字号 8px）。

#### Scenario: Numeric overlay display
- **WHEN** 单位 HP 为 85/100，EXP 为 30/50
- **THEN** 血条右侧显示 "85/100" 白色文字，经验条右侧显示 "30/50" 白色文字

### Requirement: Fixed pixel dimensions
信息条所有元素 SHALL 使用固定像素尺寸：血条 40×6px，经验条 40×4px，等级文字 10px，数值文字 8px。尺寸不随单位类型变化。

#### Scenario: Consistent bar sizes across unit types
- **WHEN** 战场上同时存在士兵和城市
- **THEN** 士兵和城市的信息条使用完全相同的像素尺寸

### Requirement: Info bar creation on first detection
系统 SHALL 在首次检测到拥有 Health 和 Level 组件的实体且满足显示条件时，创建信息条子实体并挂载 UnitInfoBar 标记组件，避免重复创建。

#### Scenario: New unit spawns
- **WHEN** 城池训练出一个新士兵
- **THEN** 系统在下一帧为该士兵创建信息条子实体，并标记 UnitInfoBar

#### Scenario: Unit already has info bar
- **WHEN** 系统检测到一个已有 UnitInfoBar 标记的实体
- **THEN** 系统仅更新其信息条子实体的显示内容（条宽度、数值文字），不创建新实体

### Requirement: System ordering
信息条渲染系统 SHALL 在 `draw_debug_shapes_system` 之后、HUD 系统之前执行，确保信息条绘制在单位几何体之上。

#### Scenario: Render order
- **WHEN** 一帧渲染时，同时存在单位几何体、信息条和屏幕空间 UI
- **THEN** 信息条渲染在单位几何体之后，但先于屏幕空间 HUD 更新

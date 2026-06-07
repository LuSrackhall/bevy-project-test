## Why

单人模式点击进入后无法正常玩耍——玩家无法选中士兵、无法下达移动/攻击指令、UI 面板不刷新、AI 不对玩家构成威胁、也没有结算界面。当前实现只是一个"模拟器"而非可操作的 RTS 游戏。需要按 ECS 调度顺序自底向上逐层修复，将游戏从"能跑"变为"可玩且完整"。

## What Changes

- **Combat 层**：修复多重射击目标选择（多支箭射不同目标）、箭矢视觉渲染、减速 debuff 叠加逻辑、近战攻击频率 bug
- **Soldier 层**：解耦士兵进城与占领逻辑、流亡士兵标记生效、减速 debuff 计时衰减与移除、出兵走廊方向动态计算
- **City 层**：实现城池点击选中、城池视觉随阵营变动刷新、中立城池被攻击翻转
- **Input 层（新模块）**：桌面端+移动端双端适配的士兵选中/框选/移动攻击指挥系统
- **UI 层**：顶部状态栏实时刷新、底部面板连接城池数据、兵种切换/举盾按钮生效、暂停按钮可用、emoji 图标全部替换为纯文本/Node 渲染（修复字体 tofu）
- **AI 层**：完善 4 项评估（防御/扩张/进攻/升级），AI 向己方士兵下达移动/攻击指令
- **Game 层**：GameOver 结算面板（胜利/失败、统计数据、再来一局/返回主菜单）

## Capabilities

### New Capabilities
- `combat-fixes`: 修复战斗系统 bug（多重射击目标、箭矢渲染、debuff 叠加、近战攻击频率）
- `soldier-system-fixes`: 修复士兵系统（进城与占领解耦、流亡士兵、debuff 衰减、出兵走廊）
- `city-interaction`: 城池点击选中、视觉刷新、中立城池翻转
- `input-selection`: 桌面+移动端双端士兵选中、框选、移动/攻击指挥（新模块 src/input/）
- `ui-system-fixes`: UI 面板数据连接、按钮功能、血条渲染、emoji 替换
- `ai-behavior`: AI 完整行为（防御/扩张/进攻/升级评估 + 士兵指挥）
- `game-over-panel`: GameOver 结算界面（统计数据、重玩/返回菜单）

### Modified Capabilities
（无——所有修改均在现有模块内部或新增模块，不改变已有 spec 级行为契约。）

## Impact

- **新增文件**：`src/input/mod.rs`（输入与选择模块）
- **修改文件**：`src/main.rs`、`src/combat/mod.rs`、`src/soldier/mod.rs`、`src/city/mod.rs`、`src/ui/hud.rs`、`src/ui/menu.rs`、`src/ui/pause.rs`、`src/ai/mod.rs`、`src/game/mod.rs`
- **新增 UI 文件**：`src/ui/gameover.rs`
- **依赖**：bevy 0.18、bevy_prototype_lyon 0.16、rand 0.8（无需变更）

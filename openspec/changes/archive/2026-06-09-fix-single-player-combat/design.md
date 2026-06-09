## Context

当前项目「城池争霸」是一个基于 Bevy 0.18 的极简 RTS 游戏。单人模式的核心游戏循环（地图生成、城池产兵、战斗伤害、AI 基础决策）已实现，但玩家无法与游戏交互——无法选中士兵、无法下达移动/攻击指令、UI 面板不刷新、AI 不指挥士兵、缺少结算界面。游戏现在只是一个"模拟器"而非可玩的 RTS。

已有代码完整——全部 9 个模块（core, map, city, soldier, combat, camera, ui, ai, game）均已实现，但存在 bug、缺失功能和未连接的数据流。

## Goals / Non-Goals

**Goals:**
- 按 ECS 调度顺序（Combat→Soldier→City→Input→UI→AI→Game）自底向上逐层修复
- 每层修复后独立可编译，不破坏已有功能
- 新增 `src/input/mod.rs` 实现 RTS 核心交互（选中+指挥），桌面端+移动端双端适配
- 所有 UI 文字以纯文本+Bevy Node 渲染，不使用 emoji（解决字体 tofu 问题）
- AI 完整实现 4 项评估并向士兵下达移动指令
- GameOver 结算面板展示统计数据

**Non-Goals:**
- 多人联机模式（标记"开发中"保持不变）
- 设置菜单、帮助菜单完整实现（保持按钮但无功能）
- Audio 音效系统
- 士兵 Sprite 替换（保持 Lyon Shape 三角形渲染）
- 移动端触摸手势的完整实现框架（仅适配层面设计，不引入 `bevy_touch` 等外部 crate）

## Decisions

### D1: 自底向上逐层修复顺序

**选择**：按 Bevy 调度顺序（Combat → Soldier → City → Input → UI → AI → Game）

**理由**：
- 底层系统修复后，上层系统可以立即利用修复后的正确行为
- 每层独立可编译验证，降低回归风险
- 设计文档第 9 节已定义此调度顺序，与其保持一致

**备选方案**：先做玩家交互再做修复
- 拒绝原因：交互依赖底层系统正确运转（如 debuff 叠加、进城逻辑），先做交互再修底层会导致重复改动

### D2: 输入系统架构

**选择**：新增 `src/input/mod.rs` 作为独立 Plugin，管理 SelectionState Resource

**理由**：
- 输入逻辑横跨士兵、城池、UI 多个模块，独立 Plugin 避免循环依赖
- 通过 Event（CitySelectedEvent）+ Resource（SelectionState）与其他模块通信
- 符合现有 ECS 架构风格（每个模块独立 Plugin）

**备选方案**：在 soldier 或 ui 模块中嵌入输入逻辑
- 拒绝原因：会导致模块职责混乱，士兵模块不该处理鼠标/触摸输入解析

### D3: 双端输入统一

**选择**：通过触碰起点检测区分"框选"和"摄像机平移"

逻辑：
- 触摸按下点命中己方单位 → 选中/框选模式
- 触摸按下点命中空地 → 摄像机平移

桌面端无需此区分（左键操作游戏、中键/右键平移摄像机）。

### D4: UI 图标方案

**选择**：全部 emoji 替换为纯文本标签 + Bevy Node 血条

**理由**：
- Bevy 的 cosmic-text 渲染器对彩色 emoji 支持有限（可能导致 tofu/方块）
- 纯文本 + Node 渲染最可靠且无外部依赖
- 血条用 Node 动态宽度实现，视觉效果更好

**备选方案**：引入 `bevy_emoji` 或加载图标字体
- 拒绝原因：增加依赖和复杂度，收益有限

### D5: AI 士兵指挥

**选择**：AI 直接修改己方 Soldier 的 `target` Component，指向目标城池 Entity

**理由**：
- 复用现有的 soldier_movement_system（士兵自动向 target 移动）
- 与玩家右键指令使用相同机制（target = Entity）
- 不需要额外的 AI-Command Event 层

## Risks / Trade-offs

- **移动端未实际测试**：移动端触摸手势仅在代码层面适配（通过窗口/触摸事件条件分支），未在真机或模拟器上测试。如后续发现 bevy_winit 在移动端的事件表现不同，可能需要调整触摸检测逻辑。
- **AI 复杂度**：完整的 4 项优先级评估涉及多轮 Query 遍历（城池+士兵），每 2 秒运行一次。在当前规模的城池数量（6-20）下性能无问题，但未来扩展需注意。
- **Waypoint Entity 方案**：右键空地生成隐形 Waypoint Entity 让士兵移动到指定位置。这些 Entity 需要在士兵到达后清理——如果士兵在到达前被杀死，Waypoint 可能泄漏。通过在 soldier_movement_system 中增加 orphan waypoint 清理解决。

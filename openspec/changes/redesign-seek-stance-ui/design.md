## Context

当前索敌 UI 在底部工具栏右侧平铺了 5 个硬编码按钮（全体开/步兵开/弓兵开/骑兵开/全关），由 `SeekBtn` 组件驱动，`seek_button_system` 处理点击。该方案存在三个问题：
1. 不支持自定义范围值（硬编码 range=10 或 0）
2. 全局指令和选区指令混为一谈，无切换逻辑
3. 无命令下发反馈，用户不知道当前生效状态

需要重新设计为结构化配置面板，支持范围输入、模式切换和命令反馈。

约束：
- Bevy 0.18 无原生文本输入框组件，需自行实现键盘输入捕获
- 索敌面板必须与现有 HUD 共存，不能破坏现有布局
- 所有命令仍通过 `CommandBuffer` + `Action::SetSeekStance` 流水线下发

## Goals / Non-Goals

**Goals:**
- 底部工具栏右侧提供索敌配置面板，替代平铺按钮
- 全局模式与选择模式在同一位置自动切换
- 兵种下拉选择框（全局/选择模式均可用）
- 范围输入框支持键盘直接输入数值
- 下发命令后顶部显示确认消息
- 选中单位时顶部显示摘要信息

**Non-Goals:**
- 不修改 `simulation` 层命令结构或消费逻辑
- 不实现复杂的消息队列/动画系统（Toast 仅覆盖显示）
- 不处理 S 键快捷命令的交互改造（保持原样）

## Decisions

### D1: 索敌面板作为独立组件区域

在底部工具栏右侧用一个分隔线隔开，放置索敌面板。面板内容根据 `SelectionState` 自动切换：
- `selected_unit_ids` 为空 → 全局模式
- `selected_unit_ids` 非空 → 选择模式

面板内部布局为水平排列：`[scope选择框] [范围输入框] [下发按钮]`。

### D2: 兵种下拉选择框实现

使用 Bevy 的 Node/Button 组件模拟下拉框：
- **触发器**：一个 Button 显示当前选中项文字 + "▼"，点击切换展开/收起
- **弹出层**：一个绝对定位的 Node 容器，包含 4 个选项 Button
- **状态**：`SeekPanelState` 资源中记录 `scope: SeekScope` 和 `dropdown_open: bool`
- 选中选项后更新 `scope` 并收起
- 点击面板外区域自动收起（通过检测 frame 内是否有 SeekScopeOption 被 pressed）

选项列表：`全体(All)` / `步兵(Infantry)` / `弓兵(Archer)` / `骑兵(Cavalry)`

### D3: 范围输入框实现

由于 Bevy 0.18 无文本输入组件，采用状态机实现：
- **显示态**：Text 组件显示当前数值，样式为带边框的输入框外观
- **编辑态**：捕获键盘 KeyCode::Digit0-9 输入追加到缓冲区，Backspace 删除末尾字符，Enter 确认并退出编辑态，Escape 取消修改并退出编辑态
- **状态**：`SeekPanelState` 资源中记录 `editing: bool`、`input_buffer: String`
- 编辑态中 `SeekPanelState.editing == true`，其他快捷键系统需检查此状态以避免冲突

默认值：全局模式 10，选择模式 30（沿用设计 D4）。

### D4: 命令下发与反馈

点击「下发」按钮时：
1. 从 `SeekPanelState` 读取当前 `scope` 和 `range`
2. 根据模式构造 `Action::SetSeekStance`
3. 推入 `CommandBuffer`
4. 更新 `ToastMessage` 资源触发顶部消息显示

全局模式命令：`SetSeekStance { scope, seek_range, unit_ids: [] }`
选择模式命令：`SetSeekStance { scope, seek_range, unit_ids: selected_unit_ids.clone() }`

### D5: 顶部消息提示（Toast）

使用 `ToastMessage` 资源存储当前消息：
- `text: String`：消息内容
- `remaining_ticks: u32`：剩余显示帧数（20Hz × 5s = 100 ticks）

系统每帧递减 `remaining_ticks`，归零时清空消息。新消息覆盖旧消息。

消息格式：
- 选中提示（单一兵种）：`选中 3 个骑兵`
- 选中提示（混合兵种）：`选中 5 个单位: 步兵2 骑兵3`
- 下发确认（全局全体）：`已下发全体索敌 范围30`
- 下发确认（全局按兵种）：`已下发步兵索敌 范围20`
- 下发确认（选中全体）：`已下发选中全体(5)索敌 范围30`
- 下发确认（选中按兵种）：`已下发选中骑兵(3)索敌 范围30`

## Risks / Trade-offs

- **Risk:** 编辑态中数字键可能与其他快捷键冲突
  → **Mitigation:** 各快捷键系统检查 `SeekPanelState.editing == false` 后再响应

- **Risk:** 下拉框弹出层可能遮挡其他 UI 元素
  → **Mitigation:** 弹出层向上展开（而非向下），避免遮挡底部工具栏下方内容

- **Trade-off:** 用状态机模拟输入框比引入第三方 UI crate 更轻量，但无法支持鼠标定位光标等高级功能
  → 这是可接受的，RTS 游戏中范围输入是低频操作，简单数字输入足够

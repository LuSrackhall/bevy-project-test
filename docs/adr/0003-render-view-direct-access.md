# ADR-003: render_view 直接访问 SimulationWorld 的临时许可

## 状态

**Superseded by ADR-006** — 阶段二已启动（simulation-command-pipeline Change）

## 决策

阶段一（DebugShape 占位阶段）允许 `render_view` 通过 `NonSendMut<SimulationWorld>` 直接读取 simulation 层数据，以及通过 `CommandBuffer` 直接注入命令。这是临时许可，不是永久架构。

## 理由

当前项目处于原型开发阶段，`presentation` 层尚未实现完整的数据中转能力。如果强制所有数据通过 `presentation` 中转，原型迭代速度会大幅下降。

## 违反的宪法条款

- §1.2.5：`render_view` 只做视觉与 UI 呈现，不得成为真相源
- §5.5：命令注入必须经过 `bevy_adapter` 的公开通道

当前 `render_view/src/selection.rs` 的 `command_issue_system` 直接写入 `simulation::CommandBuffer`，跨越两层。

## 阶段二修复路径

1. `bevy_adapter` 提供 `CommandChannel`（mpsc 或事件），`render_view` 通过通道注入命令
2. `presentation` 提供只读查询接口（`get_presentation_position(UnitId)` 等），`render_view` 通过此接口读取数据
3. 禁止 `render_view` 直接引用 `simulation` 任何类型

## 代价

阶段一的代码在阶段二需要重构。直接引用 SimulationWorld 的代码无法被 presentation 层拦截。

## 修改条件

ADR-006 生效后此 ADR 自动失效。`simulation-command-pipeline` Change 完成后，所有 `render_view` 对 `simulation` 的直接引用必须移除。

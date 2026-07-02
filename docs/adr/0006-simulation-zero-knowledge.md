# ADR-006: Simulation 对下游模块的零感知原则

## 状态

Accepted

## 决策

Simulation 层不得知道下游任何模块的存在。包括但不限于：

- Bevy（引擎框架）
- UI（按钮、面板、文本）
- Observer（事件回调）
- 输入设备（键盘、鼠标、触摸）
- 渲染管线（Camera、Transform、Sprite）
- 音频
- 网络

Simulation 与外部世界的唯一交互接口是：

```
输入：GameCommand → CommandBuffer
输出：SimulationEvents + 只读查询 (World::query)
```

## 理由

这条规则是宪法 §1 分层单向依赖的**直接推论**，不是新增约束，而是填补已有的语义缺口。

宪法 §1.1 定义了：

```
simulation ← bevy_adapter ← presentation ← render_view
```

宪法 §1.2 禁止 simulation 引入下游类型。但「不引用」不等于「不知道」。一个模块虽然不直接 import 下游类型，但仍可能被设计成「为下游服务」——这会导致隐式耦合，体现为：

1. Simulation 为 UI 特殊优化数据结构（而 UI 只是消费者之一）
2. Simulation 感知「用户操作」与「AI 操作」的区别（而它应该只消费 GameCommand）
3. 新需求引入时先问「UI 怎么显示」而非「Command 应该是什么」

这条 ADR 将宪法 §1 的原则从**编译期依赖禁令**扩展到**设计期感知禁令**。

## 违反示例

以下行为违反此 ADR：

❌ Simulation 层新增系统名称包含 `ui_`、`render_`、`input_` 前缀
❌ Simulation 层系统注释提到「为了 UI 显示」作为唯一理由
❌ Simulation 层代码中出现 `if is_player { } else { }` 而非 `match command.player_id { }`
❌ simulation Cargo.toml 中引入非白名单 bevy 子模块（§1.4）

以下行为不违反：

✅ Simulation 层使用 `SimulationEvents` 向外发布状态变更（谁消费它不知道）
✅ `consume_commands_system` 根据 `player_id` 区分命令归属（这是 GameCommand 字段，不是外部感知）
✅ Simulation 层存在 AABB 树或空间索引（这是性能优化，不是 UI 适配）

## 与 ADR-003 的关系

ADR-003 授予 `render_view` 直接访问 `SimulationWorld` 的临时许可，并明确了阶段二的修复方向。本条 ADR 是 ADR-003 阶段二的实现原则之一。当 `simulation-command-pipeline` Change 完成后，ADR-003 标记为 Superseded。

## 后续升级路径

当此原则被充分验证后（约 2-3 个迭代后），建议将其纳入宪法正文 §1，作为 §1.2 依赖禁令的补充说明，放置在 §1.2.6 位置。

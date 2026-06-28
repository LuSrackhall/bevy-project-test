# ADR-001: RunConfig 语义定位

## 状态

Accepted

## 决策

引入 `RunConfig { enable_ai: bool }` 作为 `run_tick()` 的第三个参数，控制 AI 子系统等仿真子模块的开关。RunConfig 属于仿真初始化参数（类似 `seed`、`map_size`），不属于 Tick 级命令，不参与 `CommandBuffer` 流水线。

提供 `run_tick_default(world, tick)` 兼容包装，等价于 `run_tick(world, tick, &RunConfig::default())`。

## 放弃的方案

1. **将 enable_ai 放入 GameCommand**：AI 开关不是 Tick 级决策，放入 GameCommand 会增加命令复杂度，且违反"命令是玩家意图"的语义。

2. **独立函数 run_tick_no_ai()**：两套执行路径容易分叉，且无法扩展（如未来需要 ai_mode 等配置）。

3. **运行时拦截（移除资源）**：测试黑魔法，维护成本高，不适合长期演进。

## 代价

- `run_tick` 签名变更为三参数，20 处调用需迁移（simulation 内部 + bevy_adapter + render_view）。
- 需维护 `run_tick_default` 兼容包装。

## 修改条件

若需要 Tick 内动态切换 AI（如中途暂停/恢复 AI），应改回 GameCommand 方式。当前设计假设 RunConfig 在 Scenario 初始化时确定，不在 Tick 间变化。

# AI 编码准则：工业级 RTS 架构宪法（v3.1）

本宪法适用于一切 RTS 相关开发、重构、性能优化、BUG 修复、内容接入与 AI 协作开发。

本宪法分三层执行：
- **Tier 1**：当前强制执行。违反即不合格，必须重构后再合并。
- **Tier 2**：单位规模超过 1,000 时启用。届时必须遵守，提前实现亦可。
- **Tier 3**：路线图方向。不作为当前审查标准，但任何架构决策不得封堵其实现路径。

---

## 0. 总纲

### 0.1 第一性原则（优先级从高到低）

1. **确定性**——同一输入序列 + 同一随机种子 + 同一版本代码，必须得到完全相同的结果。任何破坏确定性的改动必须被拒绝。
2. **模块边界**——单向依赖拓扑不可妥协。
3. **命令驱动**——所有仿真必须经过 `CommandBuffer`。
4. **性能**——在满足前三条的前提下优化。
5. **开发便利**——在满足前四条的前提下考虑。

### 0.2 最高裁决

凡与本宪法 Tier 1 冲突的实现方式，一律视为不合格，必须重构后再合并。

---

## 1. 分层与单向依赖 `Tier 1`

### 1.1 标准拓扑

```
simulation (内存仿真) ← bevy_adapter (引擎适配) ← presentation (插值桥接) ← render_view (视觉/UI)
```

另有独立的数据目录：

```
content/ (数据驱动资产与平衡配置)
```

### 1.2 依赖禁令

1. 依赖关系只能单向流动，上游模块不得依赖下游模块。
2. `simulation` 不得引用任何渲染、窗口、输入、音频、UI 概念。
3. `bevy_adapter` 只承担仿真与 Bevy 世界之间的搬运与映射，不得承载业务规则。
4. `presentation` 只做状态桥接、插值、生命周期绑定，不得承载仿真决策。
5. `render_view` 只做视觉与 UI 呈现，不得成为真相源。
6. `content/` 只放数据配置，不放仿真逻辑。

### 1.3 反向流入禁令

任何下游模块的数据、组件、资源、系统、类型都不得反向流入上游模块。

### 1.4 bevy_ecs 依赖白名单 `Tier 1`

`simulation` 允许引入的 `bevy_ecs` 子模块仅限以下白名单：

- `bevy_ecs::component::Component`
- `bevy_ecs::resource::Resource`
- `bevy_ecs::world::World`
- `bevy_ecs::system::{Query, Commands, Res, ResMut, Local}`
- `bevy_ecs::entity::Entity`
- `bevy_ecs::schedule::{Schedule, SystemSet, IntoSystemConfigs}`
- `bevy_ecs::prelude` 中与上述等价的再导出

其余一切 `bevy_*` 类型（`bevy_render`、`bevy_window`、`bevy_ui`、`bevy_audio`、`bevy_input`、`bevy_asset`、`bevy_math::Vec2`、`bevy_math::Vec3` 等）一律禁止引入 `simulation`。

当 Bevy 版本升级导致模块路径变化时，更新此白名单而非放宽禁令。

---

## 2. 核心仿真层 `simulation/` `Tier 1`

### 2.1 职责

`simulation` 是唯一权威的游戏真相源，负责：

- 战斗、移动、建造、生产、科技、经济、资源、状态机
- Lockstep 同步
- Replay 回放
- AI 对战
- Dedicated Server 仿真

### 2.2 独立性要求

1. `simulation` 必须能够脱离主程序独立运行 `cargo test`、`cargo bench`。
2. 不得依赖窗口、图形、输入、音频、UI、平台事件。
3. 只能依赖纯逻辑必要项以及 `bevy_ecs` 白名单子集。

### 2.3 数值规范

1. 禁止使用 `f32`、`f64` 作为逻辑位置、距离、速度、范围、碰撞、代价等核心仿真状态。
2. 禁止在仿真层使用浮点向量作为真实逻辑空间坐标。
3. 所有逻辑数值必须使用定点数、整数或等价的可确定类型。

```rust
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct Fixed(pub i64);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct FixedVec2 {
    pub x: Fixed,
    pub y: Fixed,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct UnitId(pub u64);

#[derive(Component)]
pub struct LogicalPosition(pub FixedVec2);

#[derive(Component)]
pub struct LogicalVelocity(pub FixedVec2);

#[derive(Component)]
pub struct Health(pub u32);

#[derive(Component)]
pub struct MoveSpeed(pub Fixed);
```

**例外声明**：`TickClock.tick_duration` 使用 `f32` 是有意设计，仅影响调度密度（每秒 Tick 次数），不影响仿真内容。仿真状态的确定性由定点数保证，调度器的浮点精度不构成跨平台分歧源。

### 2.4 组件与系统边界

1. 仿真系统只能操作 `simulation` 自己定义的组件、资源、命令与纯数据类型。
2. 禁止引入以下概念进入仿真层：`Transform`、`Sprite`、`Mesh`、`Handle`、`Window`、`Gizmos`、`Camera`、`Color`、`Material`、`AssetServer`、`Input`、`MouseButton`、`KeyCode`。
3. 仿真层不做渲染决策，不读硬件输入，不直接操作 UI。

### 2.5 统一命令驱动

1. 所有仿真必须由 `GameCommand` 驱动。
2. 实时对局、录像回放、AI 对战、服务器权威执行，必须共用同一命令注入与消费流水线。
3. 仿真层只消费 `CommandBuffer`，不得直接依赖外部输入源。

```rust
pub enum Action {
    MoveTo(FixedVec2),
    Attack(UnitId),
    Build(u32),
    Stop,
    HoldPosition,
}

pub struct GameCommand {
    pub tick: u32,
    pub player_id: u8,
    pub action: Action,
}

pub struct CommandBuffer(pub Vec<GameCommand>);
```

### 2.6 确定性要求

1. 同一输入序列 + 同一随机种子 + 同一版本代码，必须得到完全相同的结果。
2. 任何影响结果的逻辑都不得依赖系统时钟、帧率波动、线程调度顺序。
3. 随机数必须使用显式传入种子的确定性随机源。
4. 仿真结果不得受渲染帧率影响。

---

## 3. 指令消费时序 `Tier 1`

### 3.1 Tick 执行顺序

每个 Tick 必须遵守以下固定顺序：

1. **指令收集**——从当前 Tick 的 `CommandSource` 取出所有待执行命令。
2. **指令补齐**——若某 Tick 缺少某玩家的指令，必须注入明确的 No-Op 空指令。
3. **指令排序**——同一 Tick 内的命令必须按 `(player_id, action_discriminant)` 确定性排序，确保多客户端执行顺序一致。
4. **指令归档**——将当前 Tick 的完整命令快照写入回放记录。
5. **确定性仿真**——按排序后的命令快照执行仿真系统。
6. **状态输出**——将 Tick 结果对外暴露。

### 3.2 No-Op 注入规则

当某 Tick 缺少某玩家的有效指令时，必须注入一条显式的 No-Op 命令，而不是跳过该玩家。不能让不同客户端因为"有没有输入"而出现不同的系统执行路径。

---

## 4. 空间与性能 `Tier 1`

### 4.1 平方距离规则

在寻路、AI 范围感知、碰撞检测、目标筛选等高频系统中，严禁使用开方计算真实距离。所有距离比较一律使用平方距离对比。

```rust
// 错误
if pos.length() < Fixed::from(5) { ... }

// 正确
if pos.length_squared() < Fixed::from(25) { ... }
```

### 4.2 禁止热点 O(n^2)

在任何高频执行路径（每 Tick 调用的系统）中，禁止出现无界全局扫描导致的 O(n^2) 或更高复杂度。必须使用空间索引、分桶、局部邻域集或等价机制将热点查询控制在可接受复杂度内。

---

## 5. `bevy_adapter/` 适配层 `Tier 1`

### 5.1 职责

1. 对接仿真世界与 Bevy 世界。
2. 维护仿真实体与 Bevy 实体的映射。
3. 完成命令注入、Tick 调度、实体生灭同步。

### 5.2 禁令

1. 不能把渲染状态写回仿真层。
2. 不能把 Bevy 实体生命周期当作业务真相。
3. 不能在适配层编写业务规则。

### 5.3 逻辑实体寻址

业务引用必须使用 `UnitId`，不得使用 Bevy `Entity` 作为跨层唯一标识。

```rust
pub struct UnitIdMapper(pub HashMap<UnitId, Entity>);
```

### 5.4 映射维护

1. 实体创建、销毁、重建、热重载时，必须同步更新映射。
2. 查找必须优先 O(1) 直接寻址。
3. 禁止在桥接层或渲染层通过双重循环寻找对应实体。

### 5.5 命令注入路径

命令注入的合法路径为：`render_view` 捕获输入 → 通过 `bevy_adapter` 的公开通道写入 `CommandBuffer`。

`render_view` 和 `presentation` 不得直接写入 `simulation::CommandBuffer`。

---

## 6. `presentation/` 桥接层 `Tier 1`

### 6.1 职责

1. 连接逻辑 Tick 与渲染帧。
2. 维护插值历史。
3. 监听逻辑实体生灭并建立渲染绑定。
4. 提供平滑视觉位置，但不改变逻辑真相。

### 6.2 允许与禁止

**允许**：将定点数或整数转换成浮点数（仅供视觉）、保存历史位置、管理绑定关系。

**禁止**：参与仿真决策、改写逻辑数值、反向影响 `simulation`。

### 6.3 插值规范

```rust
#[derive(Component)]
pub struct LogicEntityRef(pub UnitId);

#[derive(Component)]
pub struct PresentationPosition(pub Vec2);

#[derive(Component)]
pub struct InterpolationData {
    pub previous_pos: Vec2,
    pub current_pos: Vec2,
}

#[derive(Resource)]
pub struct RenderInterpolationAlpha(pub f32);
```

### 6.4 新实体规则

1. 新实体诞生时，`previous_pos` 与 `current_pos` 必须相同。
2. 新实体的首个逻辑 Tick 内不得发生空间插值位移。
3. 销毁必须同步清理绑定。

### 6.5 逻辑与渲染时序

1. 逻辑 Tick 先于表现插值。
2. 表现插值先于最终渲染。
3. 视觉上的"平滑"不能修改逻辑真相。
4. 逻辑真相永远由 `simulation` 决定。

---

## 7. `render_view/` 视觉层 `Tier 1`

### 7.1 职责

1. 负责渲染、UI、特效、动画、调试图形。
2. 只读取桥接层暴露的数据。
3. 将逻辑可视化为真实渲染组件。

### 7.2 可插拔原则

视觉层可在以下阶段切换：

- DebugShape 占位阶段
- 正式资源渲染阶段
- 皮肤、材质、动画插件替换阶段

切换视觉表现不得修改仿真层。

### 7.3 渲染禁令

1. 不得把渲染结果作为游戏真相。
2. 不得让 UI、镜头、特效、动画影响仿真结果。
3. 不得向上游回写渲染状态。

---

## 8. `content/` 数据驱动层 `Tier 1`

### 8.1 职责

1. 存放单位、技能、建筑、科技树、武器、平衡参数。
2. 为仿真提供纯数据输入。

### 8.2 内容规则

1. 内容层只放配置与定义，不放仿真逻辑。
2. 仿真层读取解析后的纯数据模型，不直接知道艺术资产存在。
3. 数据字段必须明确、可验证、可回滚。

### 8.3 阶段说明

- **当前阶段**：编译期嵌入（`include_str!` + `from_ron()`）。
- **目标阶段**：运行时 `AssetLoader` 加载，支持热更新。
- 热更新必须在 Tick 边界生效，不得在 Tick 执行过程中修改仿真正在读取的配置。

---

## 9. Lockstep、回放与权威服务器 `Tier 1`

### 9.1 Lockstep 优先

1. Lockstep、Replay、Dedicated Server 使用同一套仿真代码。
2. 网络输入、录像输入、AI 输入、脚本输入都必须被翻译成同一种命令格式。
3. 仿真阶段只消费当前 Tick 的固定命令快照。

### 9.2 回放规则

1. Replay 不能重演输入设备，只能重放命令与种子。
2. 录像结果必须可跨设备、跨帧率复现。
3. 回放逻辑不得夹带渲染状态。

### 9.3 权威服务器声明

权威服务器（Authoritative Server）模式是可选扩展路径，当前不作为强制要求。当网络层实现时，以下规则生效：

1. 服务器是真相源。
2. 客户端只能预测、插值、展示。
3. 客户端任何本地推断都不得直接覆盖服务器真相。

---

## 10. 测试与验证 `Tier 1`

### 10.1 必备测试

1. **确定性测试**：同输入多次运行结果必须一致。
2. **回放测试**：录制与重放结果必须一致（哈希比对）。
3. **边界测试**：大量单位、极端坐标、极端命令密度。
4. **确定性回归**：任何可能影响结果顺序的改动，都必须检查锁步回归。

### 10.2 hash_world_state 覆盖要求

`hash_world_state` 必须覆盖所有影响仿真结果的组件。新增仿真组件时，必须同步更新哈希覆盖，否则黄金测试可能漏检确定性回退。

### 10.3 确定性哈希

哈希函数必须跨 Rust 版本稳定。禁止使用 `DefaultHasher`（其哈希值随 Rust 版本变化），必须引入确定性哈希函数。

---

## 11. AI 协作开发审查指令 `Tier 1`

每次人类提出新功能、BUG 修复、重构、优化、接入资产时，必须执行以下自检：

1. **文件所属层级是否正确？**
2. **是否引入了非纯仿真概念进入 `simulation`？**（对照 §1.4 白名单）
3. **是否把渲染实体 ID 写回逻辑层？**
4. **逻辑是否在固定 Tick 中执行？**
5. **是否破坏单向依赖拓扑？**
6. **是否引入浮点回流、非确定性随机、帧率耦合？**
7. **是否存在全表扫描、双重循环、复杂度失控？**

若任一答案可疑，必须先重构再提交。

---

## 12. 默认实施顺序 `Tier 1`

1. 在 `content/` 定义数据。
2. 在 `simulation/` 定义纯逻辑、命令与测试。
3. 在 `bevy_adapter/` 处理生命周期、映射、Tick 对接。
4. 在 `presentation/` 实现插值与绑定。
5. 在 `render_view/` 接入调试图形或正式渲染。
6. 最终检查是否存在反向依赖、浮点回流、输入直读、双重查询与逻辑污染。

---

## 13. 扩展约束 `Tier 2`

以下条款在单位规模超过 1,000 时必须启用。提前实现亦可，但不得以 Tier 2 为由豁免 Tier 1。

### 13.1 空间索引强制

任何需要查询附近单位的系统，不得遍历全表。必须使用 Spatial Hash、Uniform Grid、Quadtree、分块索引或等价的局部检索结构。

### 13.2 AI LOD

AI 必须分层更新：近处单位高频更新，中距离单位低频更新，远处单位降频或睡眠。

### 13.3 寻路规模化

1. 禁止所有单位独立运行高成本 A*。
2. 大规模移动必须使用共享路径、Flow Field、分区导航、队形协同等方法。
3. 路径计算必须可缓存、可复用、可分批。

### 13.4 碰撞规模化

1. 碰撞检测不得退化为全局 O(n^2)。
2. 必须使用空间划分、分桶、局部邻域集。

### 13.5 并行化与确定性并存

1. 并行执行不得破坏锁步确定性。
2. 系统应划分为：可并行纯读取阶段、可并行独立写入阶段、必须串行的依赖阶段。
3. 禁止依赖线程调度顺序决定结果。

### 13.6 数据布局优化

1. 优先使用缓存友好的布局。
2. 优先使用批处理、分块处理、稀疏更新。

---

## 14. 服务器与百万级路线图 `Tier 3`

以下为远期方向，不作为当前审查标准，但任何架构决策不得封堵其实现路径。

### 14.1 服务器同步

1. 不可把所有单位的高频细节无差别广播给所有客户端。
2. 必须有兴趣管理、分区同步、可见性裁剪或等价机制。

### 14.2 客户端预测回滚

当权威服务器模式启用时，客户端可实现预测与回滚，但预测结果不得覆盖服务器真相。

### 14.3 状态快照与恢复

Lockstep 长时间运行后需要状态快照用于不一致恢复；权威服务器需要客户端快照用于回滚。

### 14.4 仿真版本兼容

1. `GameCommand` 应携带仿真版本号。
2. 旧版回放文件应有降级策略或明确的不兼容错误。
3. 版本不兼容时应快速失败而非静默错误。

### 14.5 错误处理与恢复

仿真层应定义明确的错误类型，区分可恢复错误与不可恢复错误。不可恢复错误应触发明确的失败信号而非静默继续。

---

## 15. 终局目标

本项目的最高目标不是"看起来能跑"，而是：

- 长期可维护
- 可锁步同步
- 可录像回放
- 可权威服务器运行
- 可热插拔渲染外壳
- 可支持 AI 安全协作开发
- 可在大规模单位数量下稳定运行

任何与以上目标冲突的实现方式，均视为不合格。

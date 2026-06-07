# 🤖 AI 编码准则：工业级 RTS 终极架构宪法（v2.0 Final）

你正在开发一款高并发、万人同屏、支持严格锁步同步（Lockstep）、录像回放（Replay）和权威服务器（Authoritative Server）的 RTS 游戏。  
以下规则为最高级别架构宪法，任何新增功能、重构、性能优化、BUG 修复都必须严格遵守。

---

## 1. 独立模块（Crate）设计与单向依赖拓扑

项目由以下独立 Crate 组成，依赖关系只能单向流动：

`simulation (内存仿真) 💾` ← `bevy_adapter (引擎适配) ⚙️` ← `presentation (插值桥接) 🔄` ← `render_view (视觉/UI) 🎨`

另有独立的数据配置目录：

`content/ (数据驱动资产与平衡配置) 📦`

### 1.1 依赖禁令
- 严禁任何下游模块的数据、组件、资源、系统、类型反向流入上游。
- `simulation` 绝不允许引用 `presentation`、`render_view`、`bevy_window`、`bevy_render`、`bevy_ui`、`bevy_audio` 等视觉/窗口/音频概念。
- `simulation` 只能依赖纯逻辑必要项；允许使用 `bevy_ecs` 的核心子集，但禁止引入图形、窗口、输入、UI 相关 crate。
- `bevy_adapter` 负责把仿真世界与 Bevy 世界对接，但不能污染 `simulation` 的纯度。
- `presentation` 只做状态桥接、插值、生命周期绑定，不得承载业务仿真逻辑。
- `render_view` 只做视觉与 UI 呈现，不得写入核心仿真状态。

### 1.2 数据驱动资产
- 单位、技能、建筑、科技树、武器、平衡参数等一律存储在 `content/` 目录下的外置配置文件中，如 `.ron`、`.json`、`.yaml`。
- `simulation` 只能读取配置解析后的纯数据，不能知道任何艺术资产、贴图、模型、动画文件的存在。
- `simulation` 不得直接引用 `.png`、`.aseprite`、`.gltf`、`.mp3` 等资源。

---

## 2. 核心仿真层 `simulation/` —— 确定性定点数沙盒

### 2.1 架构要求
- `simulation` 必须是一个独立的、不包含任何图形、窗口、音频依赖的 Crate。
- `simulation` 必须能够脱离主程序，独立运行 `cargo test`、`cargo bench`。
- `simulation` 是权威仿真源：Lockstep、Replay、AI 对战、专用服务器都必须使用同一套仿真逻辑。
- 核心仿真运行在固定 Tick 下，例如 `20Hz` 或其它可配置固定频率。
- 所有仿真行为必须确定性可复现。

### 2.2 数值规范
- 绝对禁止在 `simulation` 中使用 `f32`、`f64` 作为空间位置、距离、速度、仿真状态数值。
- 绝对禁止在 `simulation` 中使用 `bevy_math::Vec2`、`Vec3` 等浮点向量存储逻辑位置。
- 所有空间坐标、速度、距离、范围、碰撞、寻路代价等必须使用定点数或整数。

推荐基础类型如下：

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
````

### 2.3 仿真组件与系统规范

* `simulation` 中的系统只能查询和修改 `simulation` 自己定义的组件、资源、命令与纯数据类型。
* 禁止在仿真层引入以下概念：

  * `Transform`
  * `Sprite`
  * `Mesh`
  * `Handle`
  * `Window`
  * `Gizmos`
  * `Camera`
  * `Color`
  * `Material`
  * `AssetServer`
  * `Input`
  * `MouseButton`
  * `KeyCode`
* 仿真系统必须是“逻辑纯净”的：不进行渲染决策、不读取硬件输入、不直接操作 UI。

### 2.4 Lockstep 与 Replay 一等公民

* 核心层严禁直接读取任何硬件输入、鼠标、键盘、网络包、平台事件。
* 所有仿真必须由 `GameCommand` 驱动。
* 实时对局、录像回放、AI 对战、服务器权威执行，必须使用同一套命令注入与消费流水线。
* 仿真层只消费 `CommandBuffer`，不得直接依赖外部输入源。

推荐命令模型：

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
```

推荐命令缓冲：

```rust
pub struct CommandBuffer(pub Vec<GameCommand>);
```

### 2.5 确定性要求

* 所有会影响仿真结果的逻辑必须保持确定性。
* 同一输入序列、同一随机种子、同一版本代码，必须得到完全相同的结果。
* 若使用随机数，必须使用可复现的确定性随机源，并显式传入种子。
* 不得依赖系统时钟、帧率波动、线程调度顺序来影响仿真结果。

---

## 3. 引擎适配层 `bevy_adapter/` —— 仿真世界与 Bevy 世界的唯一桥梁

### 3.1 职责

* `bevy_adapter` 是仿真层与 Bevy ECS 的对接层。
* 负责把 `simulation` 中的纯逻辑状态，映射到 Bevy 世界中可被监听、跟踪、桥接的数据。
* 负责逻辑实体的生灭同步、命令注入、Tick 调度、映射表维护。

### 3.2 适配层禁令

* 不能把渲染概念写回 `simulation`。
* 不能把渲染实体的生命周期、组件、资源当成仿真真相源。
* 不能在适配层混入业务规则，适配层只做“翻译”和“搬运”。

### 3.3 逻辑实体寻址

* 仿真层业务引用必须使用 `UnitId`，不得使用 Bevy 的 `Entity` 作为业务唯一标识。
* `Entity` 只允许作为当前 Bevy World 内部实体句柄。
* 必须在适配层维护 `UnitId -> Entity` 的 O(1) 映射资源，用于高效跨层定位。

推荐资源：

```rust
pub struct UnitIdMapper(pub HashMap<UnitId, Entity>);
```

* 实体创建、销毁、重建、热重载时，必须同步更新该映射。
* 严禁在 `presentation` 或 `render_view` 中通过双重循环或嵌套 Query 来寻找对应实体。

---

## 4. 桥接表现层 `presentation/` —— 插值、绑定、生命周期监视

### 4.1 职责

* `presentation` 负责逻辑 Tick 与渲染帧之间的时间桥接。
* 负责监听逻辑实体的生灭、建立渲染实体绑定、维护插值数据、提供平滑视觉坐标。
* 只做状态转换，不做仿真决策。

### 4.2 单向引用规范

* 渲染实体可以引用逻辑实体，但逻辑实体绝不允许引用渲染实体。
* 推荐在渲染实体上挂载逻辑引用组件：

```rust
#[derive(Component)]
pub struct LogicEntityRef(pub UnitId);
```

### 4.3 插值数据规范

* 允许在桥接层将定点数或整数逻辑位置转换为浮点数，仅供渲染使用。
* 浮点数只允许存在于 `presentation` 和 `render_view`，不得回流到 `simulation`。
* `InterpolationData` 仅保存必要的历史位置，不要把全局时间累加器放在每个实体上。

推荐组件：

```rust
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

### 4.4 生灭监听与插值规则

* 桥接层必须监听 `UnitId` 对应实体的创建与销毁。
* 当逻辑实体诞生时，必须在渲染世界中创建对应实体，并挂载 `LogicEntityRef`。
* 当逻辑实体销毁时，必须同步销毁对应的渲染实体。
* 每帧读取逻辑层的 `LogicalPosition`，结合全局 `RenderInterpolationAlpha` 计算平滑视觉位置，写入 `PresentationPosition`。
* 新实体在诞生时，必须强制将 `previous_pos` 与 `current_pos` 设为完全相同的初始值，避免首帧视觉闪烁。
* 新实体在诞生的第一个逻辑 Tick 内不进行空间位移插值。

---

## 5. 视觉与 UI 层 `render_view/` —— 可插拔的外壳

### 5.1 职责

* `render_view` 只负责视觉呈现、UI、特效、动画、调试图形。
* 只允许读取 `LogicEntityRef` 与 `PresentationPosition` 等桥接层暴露的数据。
* 只允许在这里把逻辑可视化为 Bevy 的真实渲染组件。

### 5.2 可插拔策略

* 阶段一：启用 `DebugShapePlugin`

  * 仅使用 `Gizmos`、基础几何体、方块、圆圈、线段等方式显示占位图形。
  * 不加载任何外部 `.png`、`.aseprite`、`.gltf` 资源。
* 阶段二：替换为 `Aseprite2dRenderPlugin`、`Gltf3dRenderPlugin` 或其它正式渲染插件。

  * 将 `PresentationPosition` 映射到 `Transform` 或对应渲染实体的位置。
  * 切换皮肤时不得修改 `simulation`，不得修改核心仿真数据结构。

### 5.3 渲染层禁令

* 视觉层不得把渲染状态写回逻辑层。
* 不得以渲染结果作为游戏真相。
* 不得让 UI、特效、镜头、动画影响仿真结果。

---

## 6. Content 层 `content/` —— 数据驱动资产与平衡定义

### 6.1 职责

* 所有单位、技能、建筑、科技、武器、伤害曲线、冷却时间、成本、生命值、移动速度等平衡参数必须外置。
* `content/` 只放配置与定义，不放仿真逻辑。
* 仿真层读取配置后生成内部纯数据模型，不直接依赖艺术资产。

### 6.2 内容层要求

* 内容数据必须版本化、可追踪、可热更新。
* 内容格式优先选择适合人工编辑与稳定解析的结构化格式。
* 所有数据字段必须明确、可验证、可回滚。

---

## 7. 工业级性能与同步防御性条款（Performance & Sync Guardrails）

以下规则不是建议，而是强制约束。

### 7.1 平方距离规则

* 在寻路、AI 范围感知、碰撞检测、目标筛选等高频系统中，严禁使用开方计算真实距离。
* 所有距离比较一律使用平方距离对比。

错误示例：

```rust
if pos.length() < Fixed::from(5) { ... }
```

正确示例：

```rust
if pos.length_squared() < Fixed::from(25) { ... }
```

### 7.2 指令消费时序规则

* 每个 Tick 必须先完成指令注入、补齐与归档，再进入确定性仿真阶段。
* 仿真阶段内的系统只能消费当前 Tick 的固定命令快照，不能直接读取外部输入。
* 若某个 Tick 因网络、回放或补帧缺失而没有有效指令，必须注入明确的 No-Op 空指令，保证时序稳定。
* 不能让不同客户端因为“有没有输入”而出现不同的系统执行路径。

### 7.3 新生实体视觉闪烁防御

* 新实体在中途生成时，必须在桥接层初始化插值状态，确保首帧无跳动。
* `previous_pos` 与 `current_pos` 必须在生成瞬间完全一致。
* 首个逻辑 Tick 内不得出现插值位移。

### 7.4 O(1) 跨界高效寻址规则

* 严禁在 `presentation` 或 `render_view` 中使用双重循环、全表遍历、嵌套 Query 去寻找 `UnitId` 对应的实体。
* 必须维护 O(1) 的映射资源。
* 映射资源的更新必须跟随实体生命周期同步完成。
* 查找必须优先使用 `UnitIdMapper` 之类的直接寻址结构。

---

## 8. 逻辑与渲染时序规范

### 8.1 Tick 与帧

* `simulation` 运行在固定低频 Tick 中，例如 `FixedUpdate`。
* `presentation` 运行在每帧更新阶段，用于插值与桥接。
* `render_view` 运行在渲染阶段，用于最终绘制。

### 8.2 时序要求

* 逻辑 Tick 先于表现插值。
* 表现插值先于最终渲染。
* 视觉上的“平滑”不能修改逻辑真相。
* 逻辑真相永远由 `simulation` 决定。

---

## 9. AI 协作开发审查指令

当人类要求开发新功能、修复 BUG、做重构、加性能优化、接入新资产时，必须先执行以下自检：

1. 我刚刚修改的文件属于哪个层？

   * 如果是 `simulation`，里面是否包含了 `Transform`、`Vec2`、`f32`、`Window`、`Gizmos`、`Sprite`、`Mesh`、`AssetServer` 之类的非纯仿真概念？
   * 如果有，必须立即重构并移除。

2. 我是否在逻辑组件里写入了渲染实体的 ID？

   * 正确做法是让渲染实体持有 `LogicEntityRef`，而不是让逻辑实体反向依赖渲染实体。

3. 逻辑运算是否全部在固定 Tick 中进行？

   * 仿真必须在固定 Tick 中运行。
   * 插值必须在帧更新中进行。
   * 视觉只负责显示，不负责决定结果。

4. 我是否破坏了单向依赖拓扑？

   * 上游不能依赖下游。
   * 核心仿真不能知道渲染、UI、动画、窗口、输入的存在。

5. 我是否违反了确定性与同步规则？

   * 是否使用了浮点数进入仿真？
   * 是否使用了非确定性随机源？
   * 是否让不同机器因帧率差异出现仿真分歧？

---

## 10. 最终执行原则

* 架构优先于局部便利。
* 确定性优先于视觉舒适。
* 模块边界优先于临时修补。
* 可测试、可回放、可同步、可替换，优先于短期快写。
* 任何代码若违反本宪法，必须重构后再合并。

---

## 11. 项目实施默认流程

当开始实现一个新功能时，默认遵循以下顺序：

1. 在 `content/` 中定义或更新数据配置。
2. 在 `simulation/` 中定义纯逻辑组件、命令与系统，并补齐单元测试。
3. 在 `bevy_adapter/` 中处理仿真与 Bevy 的生命周期、映射与 Tick 对接。
4. 在 `presentation/` 中实现插值、绑定、状态同步。
5. 在 `render_view/` 中接入调试图形或正式资产渲染。
6. 最后检查是否存在任何跨层反向依赖、浮点回流、输入直读、双重查询、逻辑污染。

---

## 12. 最高优先级目标

本项目的第一目标不是“看起来能跑”，而是：

* 长期可维护
* 可锁步同步
* 可录像回放
* 可权威服务器运行
* 可热插拔渲染外壳
* 可被 AI 安全协作开发
* 可在大规模单位数量下稳定运行

任何与以上目标冲突的实现方式，均视为不合格。


----

<!-- Source: superpowers-bridge/templates/adopters/CLAUDE.md.fragment.md -->
<!-- Drop this section into your project's CLAUDE.md so Claude routes future work using this schema correctly. -->
<!-- Adjust the schema name and bridge repo URL if you customized them; otherwise keep as-is. -->

## Workflow routing (read on session start)

This repo uses [`superpowers-bridge`](https://github.com/JiangWay/openspec-schemas/tree/main/superpowers-bridge) to bridge OpenSpec and Superpowers. Integration rules (language, artifact paths, PRECHECK) follow that bridge's README; this section is the routing guidance for Claude.

### Entry routing

| Trigger you observe | What to do |
|---|---|
| User starts a narrative "design discussion / let's brainstorm" | Run verbal `superpowers:brainstorming`, but **do NOT** write to `docs/superpowers/specs/`. Once the conversation converges per the 5 criteria below, promote to `/opsx:propose` |
| User invokes `/opsx:new` / `/opsx:ff` / `/opsx:propose` directly | Follow the schema's flow; artifact instructions inject at each step |
| User explicitly says bug fix / typo / config tweak / doc update | Direct PR — **do NOT** open a change (see skip rules below) |
| User is mid-change | Advance with `/opsx:continue`, `/opsx:apply`, `/opsx:verify`, or `/opsx:archive` |

### When NOT to use opsx (direct PR)

| Scenario | Direct PR? |
|---|---|
| New feature / new capability / architectural change / breaking change | ❌ Use opsx |
| Bug fix (no contract change) / test backfill / linter tweak / non-breaking upgrade / typo / docs / config value tweak | ✅ Direct PR |

Principle: **process ceremony scales with risk**. External contracts / schema / cross-system integration / compliance → opsx. Otherwise → direct PR.

### Verbal brainstorm → opsx promotion criteria

All 5 must hold before promoting (any missing → keep brainstorming, **never** write to `docs/superpowers/specs/`):

1. **Scope locked** — one sentence describes what's in / out
2. **Major design forks resolved** — alternatives weighed; remaining TBDs have an owner and impact-scope statement
3. **Cross-system dependencies mapped** — ready / mockable / genuinely unknown — pick one per dep
4. **Acceptance criteria stateable** — concrete pass conditions (e.g., `./mvnw clean verify` passes + N deliverables)
5. **Conversation converging** — recent turns are confirmations, not new alternatives

When all 5 hold → proactively suggest "ready to `/opsx:propose`?" — wait for user ack. Never auto-trigger.

### Front-door anti-patterns (don't do)

- Letting brainstorming write to `docs/superpowers/specs/`
- Letting writing-plans write to `docs/superpowers/plans/`
- Promoting to opsx with unresolved blocking TBDs
- Opening a change for bug fix / typo

Full detail: [superpowers-bridge README §Entry & exit gates](https://github.com/JiangWay/openspec-schemas/blob/main/superpowers-bridge/README.md#entry--exit-gates).

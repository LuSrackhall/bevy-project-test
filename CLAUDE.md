# AI 编码准则：工业级 RTS 架构宪法

> **权威来源：[docs/constitution.md](docs/constitution.md)（v1.0 — Frozen）**
>
> 本文件仅作为速查索引，完整条款见上方宪法正文。
> 任何与 `docs/constitution.md` 冲突的内容，以宪法正文为准。

---

## 速查：Tier 1 硬约束（违反即不合格）

### 分层拓扑

```
simulation ← bevy_adapter ← presentation ← render_view
```

依赖只能单向流动。`simulation` 不得引用任何渲染、窗口、输入、音频、UI 概念。

### simulation 禁区

禁止引入：`Transform`、`Sprite`、`Mesh`、`Handle`、`Window`、`Gizmos`、`Camera`、`Color`、`Material`、`AssetServer`、`Input`、`MouseButton`、`KeyCode`、`bevy_math::Vec2`、`bevy_math::Vec3`。

允许的 bevy_ecs 白名单：`Component`、`Resource`、`World`、`Query`、`Commands`、`Res`、`ResMut`、`Local`、`Entity`、`Schedule`、`SystemSet`。

### 数值

仿真层禁止 `f32`/`f64`，使用 `Fixed(i64)` + `FixedVec2`。距离比较一律用 `length_squared()`。

### 命令驱动

所有仿真由 `GameCommand` 驱动。同一 Tick 内命令按 `(player_id, action.sort_tag())` 排序。

### 命令注入路径

`render_view` → `bevy_adapter` 通道 → `CommandBuffer`。`render_view` 和 `presentation` 不得直接写入 `simulation::CommandBuffer`。

### 确定性

同一输入 + 同一种子 + 同一版本 = 同一结果。禁止依赖时钟、帧率、线程调度。

### Tick 时序

指令收集 → 补齐（No-Op）→ 排序 → 归档 → 仿真 → 输出。

---

## AI 自检清单（每次提交前）

1. 文件所属层级是否正确？
2. 是否引入了非纯仿真概念进入 `simulation`？（对照白名单）
3. 是否把渲染实体 ID 写回逻辑层？
4. 逻辑是否在固定 Tick 中执行？
5. 是否破坏单向依赖拓扑？
6. 是否引入浮点回流、非确定性随机、帧率耦合？
7. 是否存在全表扫描、双重循环、复杂度失控？

若任一答案可疑，必须先重构再提交。

---

## 文档体系

```
docs/
├── constitution.md      ← 架构宪法（Frozen）
├── adr/                 ← Architecture Decision Records
├── architecture/        ← 系统设计文档（随架构演进）
└── engineering/         ← 工程实践规范（编码、测试、CI）
```

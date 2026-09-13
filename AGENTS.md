# AGENTS.md — 本仓库所有 AI agent 的唯一约定源

> **单一真源（single source of truth）**
>
> - Claude Code 通过 `CLAUDE.md` 中的 `@AGENTS.md` 导入本文件 —— 因此**不要**在 `CLAUDE.md` 里编写规则。
> - 其它 agent（WorkBuddy / Codex / Cursor / Gemini / Windsurf / Zed …）请直接读本文件。
> - 修改约定**只改本文件**，改完即对所有 agent 生效。

## 目录

- [0. 开工与收工：共享工作日志](#0-开工与收工共享工作日志)
- [1. 协作纪律：git 卫生（多会话并行）](#1-协作纪律git-卫生多会话并行)
- [2. AI 编码准则：工业级 RTS 架构宪法](#2-ai-编码准则工业级-rts-架构宪法)
- [3. 技能与文档的单真源](#3-技能与文档的单真源)
- [4. 文档体系](#4-文档体系)
- [5. 构建缓存与本地环境（agent 迭代）](#5-构建缓存与本地环境agent-迭代)

---

## 0. 开工与收工：共享工作日志

本仓库的工作日志位于 `.workbuddy/memory/`。

> **目录名带 `workbuddy` 只是因为它是 WorkBuddy 的默认路径，内容是全仓共享的。**
> 任何 agent 都可以也应该往这里读写 —— 见下文的读写职责。

```
.workbuddy/memory/
├── MEMORY.md          ← 项目长期约定（就地更新，全文精简，不超过 3000 字）
└── YYYY-MM-DD.md      ← 当日工作日志（只追加，不重写）
```

### 开工前（读）

| 任务性质 | 要做的事 |
| --- | --- |
| 涉及既有决策、历史约定、前人留下的设计 | 先读 `MEMORY.md`，再读最近 1–2 个 `YYYY-MM-DD.md` |
| 跨项目、或不确定结论出处的历史 | 用会话检索，不要逐个翻日志文件 |
| 与历史无关的局部改动（如纯格式、单文件修复） | 可跳过，不必强行读 |

### 收工后（写）

完成 substantive work 后**立即**追加当日日志；当日文件不存在就创建。判断标准：

- **要记**：增删改了哪些文件及其行为变化、做出的技术选型与理由、踩到的坑与解法、遗留问题与下一步。
- **不必记**：纯问答、只读探查、单行小改、以及 git 本身就能还原的流水账。

写入要求：

1. **只追加，不重写**。新条目追加到文件末尾，不改动他人已写的条目。
2. 写之前**重读一次文件末尾**，确保追加位置基于最新内容（多会话并行时的防覆盖手段）。
3. 用统一条目格式，便于他人扫读：

```markdown
## HH:MM · <agent 名> · <一句话主题>

- **改动**：`path/to/a.rs`、`path/to/b.md` — 具体行为变化，不要只写文件名
- **决策**：选了 X 而不是 Y，因为 ……（没有决策就省略此行）
- **遗留**：未决问题 / 下一步动作（没有就省略此行）
```

4. `<agent 名>` 写自己的身份（如 `workbuddy`、`claude`、`codex`），便于事后追溯是谁做的。
5. **不写**密钥、token、凭据、个人隐私、以及任何临时性的搜索结果或报错原文。
6. 当日文件首次创建时，顶部固定写：

```markdown
# 工作日志 YYYY-MM-DD

> 多 agent 共用，只追加不重写；新条目追加在文件末尾。
```

### 长期结论写进 MEMORY.md

出现以下情况时，把结论提炼进 `MEMORY.md`（就地更新，保持精简），而不是留在当日日志里：

- 用户表达的偏好或硬性要求（例如"提交必须经我批准"）
- 项目级约定（例如"技能真源在 `.agents/skills/`"）
- 一个决策已经稳定下来、后续会话都该遵守

`MEMORY.md` 是**策展**出来的短期记忆之上的长期约束，要精炼、可执行、无冗余；超过 3000 字时应归档或删旧。

---

## 1. 协作纪律：git 卫生（多会话并行）

本工作区同时运行多个 agent 会话（也可能有人工编辑），**git 是共享资源**。

> **核心原则：一个会话只提交自己产出的路径，绝不把别人的改动卷进自己的提交。**

### 提交前

```bash
git status          # 先看清全局：哪些是本会话产物，哪些是别人的
git diff --stat     # 确认改动范围
```

### 提交时：始终用显式路径

```bash
# 1) 只暂存本会话产出的路径
git add <本会话路径>...

# 2) 提交时仍用路径限定，避免把他人已 staged 的内容一起带走
git commit -m "<type>(<scope>): <中文描述>" -- <本会话路径>...
```

第 2 步的 `-- <paths>` 不是可选项：只要工作区里存在他人已暂存的内容，`git commit` 不加路径限定就会替他们提交，污染历史且覆盖他们的提交粒度。

### 绝对禁止

- `git add -A` / `git add .` / `git add -u` / `git commit -a`
- `git stash` / `git checkout -- .` / `git reset --hard` / `git clean -fd`
- `git commit --amend` / `git rebase` / `git push --force`（改写已推送历史）
- 回滚、覆盖或丢弃其它会话的改动 —— 包括"看起来像垃圾"的已修改文件和已暂存文件

### 需要用户显式批准

- `git push`（任何分支、任何 tag）
- 合并到 `main`、执行 `git merge`、或合并 PR
- 删除分支、改写已推送历史等仓库级操作

### 隔离手段

长时间大改请开独立 worktree（`.worktrees/`、`.claude/worktrees/` 已在 `.gitignore`），不要长期占用 main 工作树，以降低与其它会话的相互干扰。

### 只读探查命令是安全的

`git status` / `git diff` / `git log` / `git show` / `git blame` 等只读命令随时可用，也是提交前的必要动作。

### 提交信息

遵循 Conventional Commits + 中文描述，参考历史风格：

```
feat(sim-cli): 增加"决出时刻"指标与 --decide-by 门禁
fix(net): 修复 Windows 双栈绑定缺陷 + 测试改走回环消除防火墙弹窗
chore(build): 构建缓存移出工作区,工作区文件数 6.6 万 → 867
```

一次会话里，代码产物与日志产物建议分成独立 commit（日志用 `docs(memory):` 前缀），使回退粒度清晰。

---

## 2. AI 编码准则：工业级 RTS 架构宪法

> **权威来源：[docs/constitution.md](docs/constitution.md)（v1.0 — Frozen）**
>
> 本节仅作为速查索引，完整条款见上方宪法正文。
> 任何与 `docs/constitution.md` 冲突的内容，以宪法正文为准。

### 速查：Tier 1 硬约束（违反即不合格）

#### 分层拓扑

```
simulation ← bevy_adapter ← presentation ← render_view
```

依赖只能单向流动。`simulation` 不得引用任何渲染、窗口、输入、音频、UI 概念。

#### simulation 禁区

禁止引入：`Transform`、`Sprite`、`Mesh`、`Handle`、`Window`、`Gizmos`、`Camera`、`Color`、`Material`、`AssetServer`、`Input`、`MouseButton`、`KeyCode`、`bevy_math::Vec2`、`bevy_math::Vec3`。

允许的 bevy_ecs 白名单：`Component`、`Resource`、`World`、`Query`、`Commands`、`Res`、`ResMut`、`Local`、`Entity`、`Schedule`、`SystemSet`。

#### 数值

仿真层禁止 `f32`/`f64`，使用 `Fixed(i64)` + `FixedVec2`。距离比较一律用 `length_squared()`。

#### 命令驱动

所有仿真由 `GameCommand` 驱动。同一 Tick 内命令按 `(player_id, action.sort_tag())` 排序。

#### 命令注入路径

`render_view` → `bevy_adapter` 通道 → `CommandBuffer`。`render_view` 和 `presentation` 不得直接写入 `simulation::CommandBuffer`。

详细实现指南（含两个 CommandBuffer 的职责区分和常见错误）：[docs/engineering/command-pipeline-guide.md](docs/engineering/command-pipeline-guide.md)

#### 确定性

同一输入 + 同一种子 + 同一版本 = 同一结果。禁止依赖时钟、帧率、线程调度。

#### Tick 时序

指令收集 → 补齐（No-Op）→ 排序 → 归档 → 仿真 → 输出。

### AI 自检清单（每次提交前）

1. 文件所属层级是否正确？
2. 是否引入了非纯仿真概念进入 `simulation`？（对照白名单）
3. 是否把渲染实体 ID 写回逻辑层？
4. 逻辑是否在固定 Tick 中执行？
5. 是否破坏单向依赖拓扑？
6. 是否引入浮点回流、非确定性随机、帧率耦合？
7. 是否存在全表扫描、双重循环、复杂度失控？

若任一答案可疑，必须先重构再提交。

---

## 3. 技能与文档的单真源

沿用本仓库的既有模式：**真源唯一，其余位置只做引用**。

| 内容 | 唯一真源 | 引用方式 |
| --- | --- | --- |
| Agent 约定 | `AGENTS.md`（本文件） | `CLAUDE.md` 用 `@AGENTS.md` 导入 |
| 技能（skills） | `.agents/skills/` | `.claude/skills` 是指向它的符号链接 |
| 架构宪法 | `docs/constitution.md` | `AGENTS.md` 只做速查索引 |

**新增或修改技能只动 `.agents/skills/`**，不要往 `.claude/skills/` 里另存一份（它是软链，会跟随真源，重复落盘即为污染）。

---

## 4. 文档体系

```
docs/
├── constitution.md      ← 架构宪法（v1.1 Active）
├── adr/                 ← Architecture Decision Records
├── architecture/        ← 系统设计文档（随架构演进）
└── engineering/         ← 工程实践规范（编码、测试、CI、Command Pipeline 实现指南）
```

---

## 5. 构建缓存与本地环境（agent 迭代）

**本机定位**：本机是**调试/迭代环境**；发布产物由 GitHub Actions 构建（`.github/workflows/release.yml`），本地不必跑 release。

**构建缓存**：`target` 位于工作区外（见 `.cargo/config.toml` 的 `target-dir`）。cargo **不回收旧代产物**——依赖版本升级、feature 集变更、profile 变更都会新增一代并永久保留：

- **结构性变更（升级依赖 / 增删 feature / 改 profile）之后跑一次 `cargo clean`**；日常改代码**不需要**（复用同一代，不会增长）。
- 实例（2026-09-13）：两天内约 10 次结构性变更累积到 66G（`bevy_ecs` 单 crate 竟有 26 代产物）；清理并把 `[profile.dev] debug` 设为 1 后稳态 **7.6G**。

**沙箱写入**：`target` 在工作区外，file 沙箱（`workspace-write`）只允许写工作区与临时目录，会拒绝写入 → 需一次性提权；若想零提权，用 `CARGO_TARGET_DIR=<工作区内路径>`（实测可行，代价是多一份缓存）。

**调试信息取舍（`[profile.dev] debug = 1`）**：出处是 Bevy 随包模板对 macOS 的建议（`.cargo/config_fast_builds.toml`）。

| 类别 | 影响 |
| --- | --- |
| 编译器诊断、panic 的 `文件:行号`、回溯的符号与行号、测试输出、`sim-cli --json`、BRP 探针 | **不受影响** |
| 交互式调试器的**局部变量值** | 退化（`opt-level` 本身也会造成部分变量不可见） |
| 需要完整变量时 | `CARGO_PROFILE_DEV_DEBUG=2 cargo run`（**只改环境变量，不改文件**） |

结论：agent 迭代依赖**结构化可观测性**（`sim-cli`、BRP 探针、黄金哈希、回放往返），不依赖交互式调试器，因此 `debug = 1` 是正确取舍；**release 与 CI（`--release`）完全不受影响**。

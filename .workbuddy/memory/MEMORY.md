# 项目长期约定（bevy-test）

> 本文件是**策展后**的项目长期约束，多 agent 共用，**就地更新**、保持精简（≤3000 字）。
> 写入时机与记录边界见 `AGENTS.md` 第 0 节。

## 协作入口（单一真源）

| 内容 | 真源 | 说明 |
| --- | --- | --- |
| Agent 约定 | `AGENTS.md` | 唯一真源；`CLAUDE.md` 只是 `@AGENTS.md` 引用壳（2026-09-13 迁移） |
| 技能 skills | `.agents/skills/` | `.claude/skills` 是指向它的符号链接，改技能只动真源 |
| 架构宪法 | `docs/constitution.md` | `AGENTS.md` 第 2 节仅为速查索引 |
| 工作日志 | `.workbuddy/memory/YYYY-MM-DD.md` | 只追加不重写；结论稳定后提炼进本文件 |

## Git 纪律（多会话并行）

- 一个会话只提交自己产出的路径；提交恒用 `git commit -- <paths>` 路径限定。
- 禁止 `git add -A` / `git add .` / `commit -a` / `stash` / `checkout -- .` / `reset --hard` / `clean -fd`。
- **禁止 `cargo fmt --all`**：它会格式化其它会话正在编辑的未提交文件。只对自己产出的路径跑 `rustfmt --edition 2021 <files>`（已实际发生：格式化扫到别人的 `remote.rs`）。
- **完成一个逻辑单元就尽快提交**：未提交的改动会被其它会话的 `git add -A` 卷进他们的提交，与提交主题无关（已实际发生：一次注释修复被卷进 `fix(net)` 提交）。
- `push`、合并到 `main`、改写已推送历史 —— 必须用户显式批准。
- 长任务开独立 worktree（`.worktrees/`、`.claude/worktrees/` 已 gitignore），别长期占用 main 工作树。
- 提交信息：Conventional Commits + 中文描述。

## 项目性质（速记）

- Rust + Bevy 0.19.x 的 RTS；依赖单向：`simulation ← bevy_adapter ← presentation ← render_view`。
- 仿真层禁浮点（用 `Fixed(i64)`/`FixedVec2`），禁渲染/输入/窗口概念，白名单见 `AGENTS.md` 第 2 节。
- 工作流：openspec（`openspec/changes`、`openspec/specs`）+ `.agents/skills/myspec-*` 技能族。
- 工具链钉在 `rust-toolchain.toml`；构建缓存已移出工作区（见 `chore(build)` 提交）。
